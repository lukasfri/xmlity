//! This module contains the [`XmlValue`] type, which is a type which can be serialized or deserialized as XML, and a type which other types can deserialize from/serialize into.
//!
//! It also contains subtypes to [`XmlValue`] including [`XmlText`], [`XmlElement`], and [`XmlAttribute`], which can be used to deserialize from/serialize into specific types of XML data.
//!
//! This is very useful in cases where you have arbitrary XML data, and you want to deserialize it into a type which can be used in your application, but you don't know what the type is ahead of time and want to be able to handle it generically.
//!
//! The types in this module can be constructed using the [`crate::xml!`] macro.
use core::{
    fmt::{self, Debug},
    str,
};
use std::{
    borrow::Cow,
    collections::VecDeque,
    fmt::Formatter,
    iter,
    ops::{Deref, DerefMut},
};

use crate::{
    de,
    ser::{self, IncludePrefix},
    types::iterator::IteratorVisitor,
    ExpandedName, ExpandedNameBuf, PrefixBuf,
};

pub mod deserialize;
mod deserializer;
mod serialize;
mod serializer;

/// Creates any `T` implementing [`Deserialize`] from an [`XmlValue`]
pub fn from_value<'de, T: crate::Deserialize<'de>>(
    value: &'de XmlValue,
) -> Result<T, XmlValueDeserializerError> {
    T::deserialize_seq(value)
}

/// Creates an [`XmlValue`] from any `T` implementing [`Serialize`].
pub fn to_value<T: crate::Serialize>(input: &T) -> Result<XmlValue, XmlValueSerializerError> {
    let mut value = XmlValue::None;
    input.serialize(&mut value)?;
    Ok(value)
}

/// A value that can be serialized or deserialized as XML, and a type which other types can deserialize from/serialize into.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum XmlValue {
    /// A text node.
    Text(XmlText),
    /// A CDATA section.
    CData(XmlCData),
    /// An element.
    Element(XmlElement),
    /// A sequence of XML values.
    Seq(XmlSeq<XmlValue>),
    /// A processing instruction.
    PI(XmlProcessingInstruction),
    /// A declaration.
    Decl(XmlDecl),
    /// A comment.
    Comment(XmlComment),
    /// A doctype.
    Doctype(XmlDoctype),
    /// Nothing.
    #[default]
    None,
}

impl From<XmlText> for XmlValue {
    fn from(value: XmlText) -> Self {
        XmlValue::Text(value)
    }
}

impl From<XmlCData> for XmlValue {
    fn from(value: XmlCData) -> Self {
        XmlValue::CData(value)
    }
}

impl From<XmlElement> for XmlValue {
    fn from(value: XmlElement) -> Self {
        XmlValue::Element(value)
    }
}

impl From<XmlSeq<XmlValue>> for XmlValue {
    fn from(value: XmlSeq<XmlValue>) -> Self {
        XmlValue::Seq(value)
    }
}

impl From<XmlProcessingInstruction> for XmlValue {
    fn from(value: XmlProcessingInstruction) -> Self {
        XmlValue::PI(value)
    }
}
impl From<XmlDecl> for XmlValue {
    fn from(value: XmlDecl) -> Self {
        XmlValue::Decl(value)
    }
}
impl From<XmlComment> for XmlValue {
    fn from(value: XmlComment) -> Self {
        XmlValue::Comment(value)
    }
}
impl From<XmlDoctype> for XmlValue {
    fn from(value: XmlDoctype) -> Self {
        XmlValue::Doctype(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
enum XmlValueWithoutSeq {
    /// A text node.
    Text(XmlText),
    /// A CDATA section.
    CData(XmlCData),
    /// An element.
    Element(XmlElement),
    /// A processing instruction.
    PI(XmlProcessingInstruction),
    /// A declaration.
    Decl(XmlDecl),
    /// A comment.
    Comment(XmlComment),
    /// A doctype.
    Doctype(XmlDoctype),
    /// Nothing.
    #[default]
    None,
}

impl From<XmlValueWithoutSeq> for XmlValue {
    fn from(value: XmlValueWithoutSeq) -> Self {
        match value {
            XmlValueWithoutSeq::Text(xml_text) => XmlValue::Text(xml_text),
            XmlValueWithoutSeq::CData(xml_cdata) => XmlValue::CData(xml_cdata),
            XmlValueWithoutSeq::Element(xml_element) => XmlValue::Element(xml_element),
            XmlValueWithoutSeq::PI(xml_processing_instruction) => {
                XmlValue::PI(xml_processing_instruction)
            }
            XmlValueWithoutSeq::Decl(xml_decl) => XmlValue::Decl(xml_decl),
            XmlValueWithoutSeq::Comment(xml_comment) => XmlValue::Comment(xml_comment),
            XmlValueWithoutSeq::Doctype(xml_doctype) => XmlValue::Doctype(xml_doctype),
            XmlValueWithoutSeq::None => XmlValue::None,
        }
    }
}

/// A text node in an XML document.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct XmlText(pub Vec<u8>);

impl XmlText {
    /// Creates a new [`XmlText`] from a string.
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        Self(text.as_ref().as_bytes().to_vec())
    }
}

impl Debug for XmlText {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("XmlText")
            .field(&String::from_utf8_lossy(&self.0))
            .finish()
    }
}

impl From<String> for XmlText {
    fn from(value: String) -> Self {
        Self(value.into_bytes())
    }
}

impl From<&str> for XmlText {
    fn from(value: &str) -> Self {
        Self(value.to_owned().into_bytes())
    }
}

/// CDATA section.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct XmlCData(pub Vec<u8>);

impl XmlCData {
    /// Creates a new [CDATA section](`XmlCData`).
    pub fn new<T: Into<Vec<u8>>>(text: T) -> Self {
        Self(text.into())
    }
}

/// An XML child node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum XmlChild {
    /// A text node.
    Text(XmlText),
    /// A CDATA node.
    CData(XmlCData),
    /// An element node.
    Element(XmlElement),
    /// A processing instruction node.
    PI(XmlProcessingInstruction),
    /// A comment node.
    Comment(XmlComment),
    /// Nothing.
    #[default]
    None,
}

impl From<XmlText> for XmlChild {
    fn from(value: XmlText) -> Self {
        XmlChild::Text(value)
    }
}

impl From<XmlCData> for XmlChild {
    fn from(value: XmlCData) -> Self {
        XmlChild::CData(value)
    }
}

impl From<XmlElement> for XmlChild {
    fn from(value: XmlElement) -> Self {
        XmlChild::Element(value)
    }
}

impl From<XmlProcessingInstruction> for XmlChild {
    fn from(value: XmlProcessingInstruction) -> Self {
        XmlChild::PI(value)
    }
}

impl From<XmlComment> for XmlChild {
    fn from(value: XmlComment) -> Self {
        XmlChild::Comment(value)
    }
}

/// An XML element.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct XmlElement {
    /// The name of the element.
    pub name: ExpandedNameBuf,
    /// The attributes of the element.
    pub attributes: VecDeque<XmlAttribute>,
    /// The children of the element.
    pub children: XmlSeq<XmlChild>,
    /// Whether to enforce the prefix of the element.
    pub enforce_prefix: IncludePrefix,
    /// The preferred prefix of the element.
    pub preferred_prefix: Option<PrefixBuf>,
}

impl XmlElement {
    /// Creates a new XML element.
    pub fn new<T: Into<ExpandedNameBuf>>(name: T) -> Self {
        Self {
            name: name.into(),
            attributes: VecDeque::new(),
            children: XmlSeq::new(),
            enforce_prefix: IncludePrefix::default(),
            preferred_prefix: None,
        }
    }

    /// Adds an attribute to the element.
    pub fn with_attribute<T: Into<XmlAttribute>>(mut self, attribute: T) -> Self {
        self.attributes.push_back(attribute.into());
        self
    }

    /// Adds multiple attributes to the element.
    pub fn with_attributes<U: Into<XmlAttribute>, T: IntoIterator<Item = U>>(
        mut self,
        attributes: T,
    ) -> Self {
        self.attributes
            .extend(attributes.into_iter().map(Into::into));
        self
    }

    /// Adds a child to the element.
    pub fn with_child<T: Into<XmlChild>>(mut self, child: T) -> Self {
        self.children.values.push_back(child.into());
        self
    }

    /// Adds multiple children to the element.
    pub fn with_children<U: Into<XmlChild>, T: IntoIterator<Item = U>>(
        mut self,
        children: T,
    ) -> Self {
        self.children
            .values
            .extend(children.into_iter().map(Into::into));
        self
    }
}

/// An XML attribute.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct XmlAttribute {
    /// The name of the attribute.
    pub name: ExpandedNameBuf,
    /// The value of the attribute.
    pub value: XmlText,
}

impl XmlAttribute {
    /// Creates a new XML attribute.
    pub fn new<T: Into<ExpandedNameBuf>, U: Into<XmlText>>(name: T, value: U) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// A sequence of XML elements.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct XmlSeq<T> {
    values: VecDeque<T>,
}

impl<T> IntoIterator for XmlSeq<T> {
    type Item = T;
    type IntoIter = std::collections::vec_deque::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<T> FromIterator<T> for XmlSeq<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            values: iter.into_iter().collect(),
        }
    }
}

impl<T> From<VecDeque<T>> for XmlSeq<T> {
    fn from(value: VecDeque<T>) -> Self {
        Self::from_vec_deque(value)
    }
}

impl<T> XmlSeq<T> {
    /// Creates a new empty sequence.
    pub fn new() -> Self {
        Self::from_vec_deque(VecDeque::new())
    }

    /// Creates a new sequence from a [`VecDeque<T>`].
    pub fn from_vec_deque(values: VecDeque<T>) -> Self {
        Self { values }
    }

    /// Gets the inner [`VecDeque<T>`] of the sequence.
    pub fn into_inner(self) -> VecDeque<T> {
        self.values
    }
}

impl Deref for XmlSeq<XmlValue> {
    type Target = VecDeque<XmlValue>;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl DerefMut for XmlSeq<XmlValue> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}

impl<T> Default for XmlSeq<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A processing instruction.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct XmlProcessingInstruction {
    target: Vec<u8>,
    content: Vec<u8>,
}

impl XmlProcessingInstruction {
    /// Creates a new processing instruction.
    pub fn new<T: Into<Vec<u8>>, U: Into<Vec<u8>>>(target: T, content: U) -> Self {
        Self {
            target: target.into(),
            content: content.into(),
        }
    }
}

/// Represents an XML declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct XmlDecl {
    /// The version of the XML document.
    pub version: String,
    /// The encoding of the XML document.
    pub encoding: Option<String>,
    /// The standalone status of the XML document.
    pub standalone: Option<String>,
}

impl XmlDecl {
    /// Creates a new XML declaration.
    pub fn new<T: AsRef<str>>(version: T, encoding: Option<T>, standalone: Option<T>) -> Self {
        Self {
            version: version.as_ref().to_string(),
            encoding: encoding.map(|e| e.as_ref().to_string()),
            standalone: standalone.map(|s| s.as_ref().to_string()),
        }
    }
}

/// XML Comment
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct XmlComment(pub Vec<u8>);

impl XmlComment {
    /// Creates a new XML comment.
    pub fn new<T: Into<Vec<u8>>>(comment: T) -> Self {
        Self(comment.into())
    }
}

/// A doctype declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct XmlDoctype(pub Vec<u8>);

impl XmlDoctype {
    /// Creates a new doctype declaration.
    pub fn new<T: Into<Vec<u8>>>(value: T) -> Self {
        Self(value.into())
    }
}

/// Error type for serializing XML values.
#[derive(Debug, thiserror::Error)]
pub enum XmlValueSerializerError {
    /// Error for when a custom error occurs during serialization.
    #[error("Custom error: {0}")]
    Custom(String),
    /// Error for when an unexpected serialization occurs.
    #[error("Unexpected serialization: {0}")]
    UnexpectedSerialize(ser::Unexpected),
}

impl ser::Error for XmlValueSerializerError {
    fn unexpected_serialize(unexpected: ser::Unexpected) -> Self {
        Self::UnexpectedSerialize(unexpected)
    }

    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

/// Error type for deserializing XML values.
#[derive(Debug, thiserror::Error)]
pub enum XmlValueDeserializerError {
    /// Error for when an unexpected visit occurs during deserialization.
    #[error("Unexpected visit: {0}")]
    UnexpectedVisit(crate::de::Unexpected),
    /// Error for when a custom error occurs during deserialization.
    #[error("Custom error: {0}")]
    Custom(String),
    /// Error for when a name is expected to be a certain value, but it is not.
    #[error("Wrong name: {actual:?}, expected: {expected:?}")]
    WrongName {
        /// The actual name that was encountered.
        actual: Box<ExpandedNameBuf>,
        /// The expected name.
        expected: Box<ExpandedNameBuf>,
    },
    /// Error for when a field is missing.
    #[error("Missing field: {0}")]
    MissingField(String),
    /// Error for when a child cannot be identified, and ignoring it is not allowed.
    #[error("Unknown child")]
    UnknownChild,
    /// Error for when a string is invalid for the type.
    #[error("Invalid string")]
    InvalidString,
    /// Error for when a type has no possible variants to deserialize into.
    #[error("No possible variant")]
    NoPossibleVariant {
        /// The name of the type that has no possible variants.
        ident: String,
    },
    /// Error for when a type is missing data that is required to deserialize it.
    #[error("Missing data")]
    MissingData,
}

impl de::Error for XmlValueDeserializerError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }

    fn wrong_name(name: &ExpandedName<'_>, expected: &ExpandedName<'_>) -> Self {
        Self::WrongName {
            actual: Box::new(name.into_owned()),
            expected: Box::new(expected.into_owned()),
        }
    }

    fn unexpected_visit<T>(unexpected: de::Unexpected, _expected: &T) -> Self {
        Self::UnexpectedVisit(unexpected)
    }

    fn missing_field(field: &str) -> Self {
        Self::MissingField(field.to_string())
    }

    fn no_possible_variant(ident: &str) -> Self {
        Self::NoPossibleVariant {
            ident: ident.to_string(),
        }
    }

    fn missing_data() -> Self {
        Self::MissingData
    }

    fn unknown_child() -> Self {
        Self::UnknownChild
    }

    fn invalid_string() -> Self {
        Self::InvalidString
    }
}
