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
use std::{borrow::Cow, fmt::Formatter, iter, ops::Deref};

use crate::{
    de::{self, AttributesAccess, ElementAccess, NamespaceContext, SeqAccess, Visitor},
    ser::{
        self, IncludePrefix, SerializeAttributeAccess, SerializeAttributes, SerializeElement,
        SerializeElementAttributes, SerializeSeq,
    },
    AttributeSerializer, Deserialize, Deserializer, ExpandedName, Prefix, Serialize,
    SerializeAttribute, Serializer, XmlNamespace,
};

use super::iterator::IteratorVisitor;

/// Creates any `T` implementing [`Deserialize`] from an [`XmlValue`]
pub fn from_value<'de, T: Deserialize<'de>>(
    value: &'de XmlValue,
) -> Result<T, XmlValueDeserializerError> {
    T::deserialize_seq(value)
}

/// Creates an [`XmlValue`] from any `T` implementing [`Serialize`].
pub fn to_value<T: Serialize>(input: &T) -> Result<XmlValue, XmlValueSerializerError> {
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

impl Serialize for XmlValue {
    fn serialize<S: crate::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            XmlValue::Text(xml_text) => xml_text.serialize(serializer),
            XmlValue::CData(xml_cdata) => xml_cdata.serialize(serializer),
            XmlValue::Element(xml_element) => xml_element.serialize(serializer),
            XmlValue::Seq(xml_seq) => xml_seq.serialize(serializer),
            XmlValue::PI(xml_pi) => xml_pi.serialize(serializer),
            XmlValue::Decl(xml_decl) => xml_decl.serialize(serializer),
            XmlValue::Comment(xml_comment) => xml_comment.serialize(serializer),
            XmlValue::Doctype(xml_doctype) => xml_doctype.serialize(serializer),
            XmlValue::None => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for XmlValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct __Visitor<'v> {
            marker: ::core::marker::PhantomData<XmlValue>,
            lifetime: ::core::marker::PhantomData<&'v ()>,
        }

        impl<'v> crate::de::Visitor<'v> for __Visitor<'v> {
            type Value = XmlValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a comment")
            }

            fn visit_text<E, V>(self, value: V) -> Result<Self::Value, E>
            where
                E: de::Error,
                V: de::XmlText<'v>,
            {
                XmlTextVisitor::new().visit_text(value).map(XmlValue::Text)
            }

            fn visit_cdata<E, V>(self, value: V) -> Result<Self::Value, E>
            where
                E: de::Error,
                V: de::XmlCData,
            {
                XmlCDataVisitor::new()
                    .visit_cdata(value)
                    .map(XmlValue::CData)
            }

            fn visit_element<A>(self, element: A) -> Result<Self::Value, A::Error>
            where
                A: de::ElementAccess<'v>,
            {
                XmlElementVisitor::new()
                    .visit_element(element)
                    .map(XmlValue::Element)
            }

            fn visit_seq<S>(self, sequence: S) -> Result<Self::Value, S::Error>
            where
                S: de::SeqAccess<'v>,
            {
                IteratorVisitor::<_, XmlSeq<XmlValue>>::default()
                    .visit_seq(sequence)
                    .map(XmlValue::Seq)
            }

            fn visit_pi<E, V>(self, value: V) -> Result<Self::Value, E>
            where
                E: de::Error,
                V: de::XmlProcessingInstruction,
            {
                XmlProcessingInstructionVisitor::new()
                    .visit_pi(value)
                    .map(XmlValue::PI)
            }

            fn visit_decl<E, V>(self, declaration: V) -> Result<Self::Value, E>
            where
                E: de::Error,
                V: de::XmlDeclaration,
            {
                XmlDeclVisitor::new()
                    .visit_decl(declaration)
                    .map(XmlValue::Decl)
            }

            fn visit_comment<E, V>(self, comment: V) -> Result<Self::Value, E>
            where
                E: de::Error,
                V: de::XmlComment,
            {
                XmlCommentVisitor::new()
                    .visit_comment(comment)
                    .map(XmlValue::Comment)
            }

            fn visit_doctype<E, V>(self, value: V) -> Result<Self::Value, E>
            where
                E: de::Error,
                V: de::XmlDoctype,
            {
                XmlDoctypeVisitor::new()
                    .visit_doctype(value)
                    .map(XmlValue::Doctype)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(XmlValue::None)
            }
        }

        deserializer.deserialize_any(__Visitor {
            lifetime: ::core::marker::PhantomData,
            marker: ::core::marker::PhantomData,
        })
    }
}

impl<'s> Serializer for &'s mut &mut XmlSeq<XmlValue> {
    type Ok = ();
    type Error = XmlValueSerializerError;

    type SerializeSeq = &'s mut XmlSeq<XmlValue>;
    type SerializeElement = &'s mut XmlElement;

    fn serialize_cdata<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.values
            .push(XmlValue::CData(XmlCData::new(text.as_ref().as_bytes())));
        Ok(())
    }

    fn serialize_text<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.values.push(XmlValue::Text(XmlText::new(text)));
        Ok(())
    }

    fn serialize_element(
        self,
        name: &'_ ExpandedName<'_>,
    ) -> Result<Self::SerializeElement, Self::Error> {
        self.values.push(XmlValue::Element(XmlElement::new(
            name.clone().into_owned(),
        )));

        let XmlValue::Element(element) = self.values.last_mut().expect("just pushed") else {
            unreachable!()
        };

        Ok(element)
    }

    fn serialize_seq(self) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_pi<S: AsRef<[u8]>>(self, target: S, content: S) -> Result<Self::Ok, Self::Error> {
        self.values.push(XmlValue::PI(XmlProcessingInstruction {
            target: target.as_ref().to_vec(),
            content: content.as_ref().to_vec(),
        }));
        Ok(())
    }

    fn serialize_comment<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.values
            .push(XmlValue::Comment(XmlComment(text.as_ref().to_vec())));
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.values.push(XmlValue::None);
        Ok(())
    }

    fn serialize_decl<S: AsRef<str>>(
        self,
        version: S,
        encoding: Option<S>,
        standalone: Option<S>,
    ) -> Result<Self::Ok, Self::Error> {
        self.values
            .push(XmlValue::Decl(XmlDecl::new(version, encoding, standalone)));
        Ok(())
    }

    fn serialize_doctype<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.values
            .push(XmlValue::Doctype(XmlDoctype(text.as_ref().to_vec())));
        Ok(())
    }
}

impl<'s> Serializer for &'s mut XmlValue {
    type Ok = ();
    type Error = XmlValueSerializerError;

    type SerializeSeq = &'s mut XmlSeq<XmlValue>;
    type SerializeElement = &'s mut XmlElement;

    fn serialize_cdata<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        *self = XmlValue::CData(XmlCData::new(text.as_ref().as_bytes()));
        Ok(())
    }

    fn serialize_text<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        *self = XmlValue::Text(XmlText::new(text));
        Ok(())
    }

    fn serialize_element(
        self,
        name: &'_ ExpandedName<'_>,
    ) -> Result<Self::SerializeElement, Self::Error> {
        *self = XmlValue::Element(XmlElement::new(name.clone().into_owned()));

        let XmlValue::Element(element) = self else {
            unreachable!()
        };

        Ok(element)
    }

    fn serialize_seq(self) -> Result<Self::SerializeSeq, Self::Error> {
        *self = XmlValue::Seq(XmlSeq::new());
        let XmlValue::Seq(seq) = self else {
            unreachable!()
        };
        Ok(seq)
    }

    fn serialize_decl<S: AsRef<str>>(
        self,
        version: S,
        encoding: Option<S>,
        standalone: Option<S>,
    ) -> Result<Self::Ok, Self::Error> {
        *self = XmlValue::Decl(XmlDecl::new(version, encoding, standalone));
        Ok(())
    }

    fn serialize_pi<S: AsRef<[u8]>>(self, target: S, content: S) -> Result<Self::Ok, Self::Error> {
        *self = XmlValue::PI(XmlProcessingInstruction {
            target: target.as_ref().to_vec(),
            content: content.as_ref().to_vec(),
        });
        Ok(())
    }

    fn serialize_comment<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        *self = XmlValue::Comment(XmlComment(text.as_ref().to_vec()));
        Ok(())
    }

    fn serialize_doctype<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        *self = XmlValue::Doctype(XmlDoctype(text.as_ref().to_vec()));
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        *self = XmlValue::None;
        Ok(())
    }
}

impl<'de> Deserializer<'de> for &'de XmlValue {
    type Error = XmlValueDeserializerError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            XmlValue::Text(xml_text) => xml_text.deserialize_any(visitor),
            XmlValue::CData(xml_cdata) => xml_cdata.deserialize_any(visitor),
            XmlValue::Element(xml_element) => xml_element.deserialize_any(visitor),
            XmlValue::Seq(xml_seq) => XmlSeqAccess::new(xml_seq).deserialize_any(visitor),
            XmlValue::PI(xml_pi) => xml_pi.deserialize_any(visitor),
            XmlValue::Decl(xml_decl) => xml_decl.deserialize_any(visitor),
            XmlValue::Comment(xml_comment) => xml_comment.deserialize_any(visitor),
            XmlValue::Doctype(xml_doctype) => xml_doctype.deserialize_any(visitor),
            XmlValue::None => visitor.visit_none(),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            XmlValue::Text(xml_text) => xml_text.deserialize_seq(visitor),
            XmlValue::CData(xml_cdata) => xml_cdata.deserialize_seq(visitor),
            XmlValue::Element(xml_element) => xml_element.deserialize_seq(visitor),
            XmlValue::Seq(xml_seq) => XmlSeqAccess::new(xml_seq).deserialize_seq(visitor),
            XmlValue::PI(xml_pi) => xml_pi.deserialize_seq(visitor),
            XmlValue::Decl(xml_decl) => xml_decl.deserialize_seq(visitor),
            XmlValue::Comment(xml_comment) => xml_comment.deserialize_seq(visitor),
            XmlValue::Doctype(xml_doctype) => xml_doctype.deserialize_seq(visitor),
            XmlValue::None => visitor.visit_none(),
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

impl Serialize for XmlText {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_text(String::from_utf8_lossy(&self.0))
    }
}

struct XmlTextVisitor<'v> {
    marker: ::core::marker::PhantomData<XmlText>,
    lifetime: ::core::marker::PhantomData<&'v ()>,
}

impl XmlTextVisitor<'_> {
    fn new() -> Self {
        Self {
            marker: ::core::marker::PhantomData,
            lifetime: ::core::marker::PhantomData,
        }
    }
}

impl<'v> crate::de::Visitor<'v> for XmlTextVisitor<'v> {
    type Value = XmlText;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a comment")
    }

    fn visit_text<E, V>(self, value: V) -> Result<Self::Value, E>
    where
        E: de::Error,
        V: de::XmlText<'v>,
    {
        Ok(XmlText(value.into_bytes().into()))
    }
}

impl<'de> Deserialize<'de> for XmlText {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(XmlTextVisitor::new())
    }
}

impl NamespaceContext for () {
    fn resolve_prefix(&self, _prefix: Prefix<'_>) -> Option<XmlNamespace<'_>> {
        None
    }
}

impl<'de> de::XmlText<'de> for &'de XmlText {
    type NamespaceContext<'a>
        = ()
    where
        Self: 'a;

    fn into_bytes(self) -> Cow<'de, [u8]> {
        Cow::Borrowed(&self.0)
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn into_string(self) -> Cow<'de, str> {
        Cow::Borrowed(std::str::from_utf8(&self.0).unwrap())
    }

    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap()
    }

    fn namespace_context(&self) -> Self::NamespaceContext<'_> {}
}

impl<'de> de::SeqAccess<'de> for Option<&'de XmlText> {
    type Error = XmlValueDeserializerError;

    type SubAccess<'g>
        = Self
    where
        Self: 'g;

    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        let Some(text) = self.take() else {
            return Ok(None);
        };

        match T::deserialize(text) {
            Ok(value) => Ok(Some(value)),
            Err(_) => {
                *self = Some(text);
                Ok(None)
            }
        }
    }

    fn next_element_seq<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        let Some(text) = self.take() else {
            return Ok(None);
        };

        match T::deserialize_seq(text) {
            Ok(value) => Ok(Some(value)),
            Err(_) => {
                *self = Some(text);
                Ok(None)
            }
        }
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        Ok(*self)
    }
}

impl<'de> Deserializer<'de> for &'de XmlText {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_text(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(Some(self))
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

impl Serialize for XmlCData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_cdata(str::from_utf8(&self.0).unwrap())
    }
}

struct XmlCDataVisitor<'v> {
    marker: ::core::marker::PhantomData<XmlCData>,
    lifetime: ::core::marker::PhantomData<&'v ()>,
}
impl XmlCDataVisitor<'_> {
    fn new() -> Self {
        Self {
            marker: ::core::marker::PhantomData,
            lifetime: ::core::marker::PhantomData,
        }
    }
}

impl<'de> Visitor<'de> for XmlCDataVisitor<'de> {
    type Value = XmlCData;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a CDATA section")
    }
    fn visit_cdata<E, V>(self, value: V) -> Result<Self::Value, E>
    where
        E: de::Error,
        V: de::XmlCData,
    {
        Ok(XmlCData(value.as_bytes().to_owned()))
    }
}

impl<'de> Deserialize<'de> for XmlCData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(XmlCDataVisitor::new())
    }
}

impl de::XmlCData for &XmlCData {
    type NamespaceContext<'a>
        = ()
    where
        Self: 'a;

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn as_str(&self) -> Cow<'_, str> {
        Cow::Borrowed(std::str::from_utf8(&self.0).unwrap())
    }

    fn namespace_context(&self) -> Self::NamespaceContext<'_> {}
}

impl<'de> Deserializer<'de> for &'de XmlCData {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_cdata(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
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

impl Serialize for XmlChild {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            XmlChild::Text(v) => v.serialize(serializer),
            XmlChild::CData(v) => v.serialize(serializer),
            XmlChild::Element(v) => v.serialize(serializer),
            XmlChild::PI(v) => v.serialize(serializer),
            XmlChild::Comment(v) => v.serialize(serializer),
            XmlChild::None => serializer.serialize_none(),
        }
    }
}

struct XmlChildVisitor<'v> {
    marker: ::core::marker::PhantomData<XmlChild>,
    lifetime: ::core::marker::PhantomData<&'v ()>,
}
impl XmlChildVisitor<'_> {
    fn new() -> Self {
        Self {
            marker: ::core::marker::PhantomData,
            lifetime: ::core::marker::PhantomData,
        }
    }
}

impl<'v> crate::de::Visitor<'v> for XmlChildVisitor<'v> {
    type Value = XmlChild;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an XML child")
    }

    fn visit_text<E, V>(self, value: V) -> Result<Self::Value, E>
    where
        E: de::Error,
        V: de::XmlText<'v>,
    {
        XmlTextVisitor::new().visit_text(value).map(XmlChild::Text)
    }

    fn visit_cdata<E, V>(self, value: V) -> Result<Self::Value, E>
    where
        E: de::Error,
        V: de::XmlCData,
    {
        XmlCDataVisitor::new()
            .visit_cdata(value)
            .map(XmlChild::CData)
    }

    fn visit_element<A>(self, element: A) -> Result<Self::Value, A::Error>
    where
        A: de::ElementAccess<'v>,
    {
        XmlElementVisitor::new()
            .visit_element(element)
            .map(XmlChild::Element)
    }

    fn visit_pi<E, V>(self, value: V) -> Result<Self::Value, E>
    where
        E: de::Error,
        V: de::XmlProcessingInstruction,
    {
        XmlProcessingInstructionVisitor::new()
            .visit_pi(value)
            .map(XmlChild::PI)
    }

    fn visit_comment<E, V>(self, value: V) -> Result<Self::Value, E>
    where
        E: de::Error,
        V: de::XmlComment,
    {
        XmlCommentVisitor::new()
            .visit_comment(value)
            .map(XmlChild::Comment)
    }
}

impl<'de> de::Deserialize<'de> for XmlChild {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(XmlChildVisitor::new())
    }
}

impl<'s> Serializer for &'s mut &mut XmlSeq<XmlChild> {
    type Ok = ();
    type Error = XmlValueSerializerError;

    type SerializeSeq = &'s mut XmlSeq<XmlChild>;
    type SerializeElement = &'s mut XmlElement;

    fn serialize_cdata<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.values
            .push(XmlChild::CData(XmlCData::new(text.as_ref().as_bytes())));
        Ok(())
    }

    fn serialize_text<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.values.push(XmlChild::Text(XmlText::new(text)));
        Ok(())
    }

    fn serialize_element(
        self,
        name: &'_ ExpandedName<'_>,
    ) -> Result<Self::SerializeElement, Self::Error> {
        self.values.push(XmlChild::Element(XmlElement::new(
            name.clone().into_owned(),
        )));

        let XmlChild::Element(element) = self.values.last_mut().expect("just pushed") else {
            unreachable!()
        };

        Ok(element)
    }

    fn serialize_seq(self) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_pi<S: AsRef<[u8]>>(self, target: S, content: S) -> Result<Self::Ok, Self::Error> {
        self.values.push(XmlChild::PI(XmlProcessingInstruction {
            target: target.as_ref().to_vec(),
            content: content.as_ref().to_vec(),
        }));
        Ok(())
    }

    fn serialize_comment<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error> {
        self.values
            .push(XmlChild::Comment(XmlComment(text.as_ref().to_vec())));
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.values.push(XmlChild::None);
        Ok(())
    }

    fn serialize_decl<S: AsRef<str>>(
        self,
        _version: S,
        _encoding: Option<S>,
        _standalone: Option<S>,
    ) -> Result<Self::Ok, Self::Error> {
        Err(XmlValueSerializerError::InvalidChildDeserialization)
    }

    fn serialize_doctype<S: AsRef<[u8]>>(self, _text: S) -> Result<Self::Ok, Self::Error> {
        Err(XmlValueSerializerError::InvalidChildDeserialization)
    }
}

impl<'de> Deserializer<'de> for &'de XmlChild {
    type Error = XmlValueDeserializerError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            XmlChild::Text(xml_text) => xml_text.deserialize_any(visitor),
            XmlChild::CData(xml_cdata) => xml_cdata.deserialize_any(visitor),
            XmlChild::Element(xml_element) => xml_element.deserialize_any(visitor),
            XmlChild::PI(xml_pi) => xml_pi.deserialize_any(visitor),
            XmlChild::Comment(xml_comment) => xml_comment.deserialize_any(visitor),
            XmlChild::None => visitor.visit_none(),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            XmlChild::Text(xml_text) => xml_text.deserialize_seq(visitor),
            XmlChild::CData(xml_cdata) => xml_cdata.deserialize_seq(visitor),
            XmlChild::Element(xml_element) => xml_element.deserialize_seq(visitor),
            XmlChild::PI(xml_pi) => xml_pi.deserialize_seq(visitor),
            XmlChild::Comment(xml_comment) => xml_comment.deserialize_seq(visitor),
            XmlChild::None => visitor.visit_none(),
        }
    }
}

/// An XML element.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct XmlElement {
    /// The name of the element.
    pub name: ExpandedName<'static>,
    /// The attributes of the element.
    pub attributes: Vec<XmlAttribute>,
    /// The children of the element.
    pub children: XmlSeq<XmlChild>,
    /// Whether to enforce the prefix of the element.
    pub enforce_prefix: IncludePrefix,
    /// The preferred prefix of the element.
    pub preferred_prefix: Option<Prefix<'static>>,
}

impl XmlElement {
    /// Creates a new XML element.
    pub fn new<T: Into<ExpandedName<'static>>>(name: T) -> Self {
        Self {
            name: name.into(),
            attributes: Vec::new(),
            children: XmlSeq::new(),
            enforce_prefix: IncludePrefix::default(),
            preferred_prefix: None,
        }
    }

    /// Adds an attribute to the element.
    pub fn with_attribute<T: Into<XmlAttribute>>(mut self, attribute: T) -> Self {
        self.attributes.push(attribute.into());
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
        self.children.values.push(child.into());
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

impl Serialize for XmlElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let element = serializer.serialize_element(&self.name)?;

        let mut attributes = element.serialize_attributes()?;
        for attr in &self.attributes {
            attributes.serialize_attribute(attr)?;
        }

        if self.children.values.is_empty() {
            return attributes.end();
        }

        let mut children = attributes.serialize_children()?;
        for child in &self.children.values {
            children.serialize_element(child)?;
        }
        children.end()
    }
}

struct XmlElementVisitor<'v> {
    marker: ::core::marker::PhantomData<XmlElement>,
    lifetime: ::core::marker::PhantomData<&'v ()>,
}
impl XmlElementVisitor<'_> {
    fn new() -> Self {
        Self {
            marker: ::core::marker::PhantomData,
            lifetime: ::core::marker::PhantomData,
        }
    }
}

impl<'v> crate::de::Visitor<'v> for XmlElementVisitor<'v> {
    type Value = XmlElement;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an element")
    }

    fn visit_element<A>(self, mut element: A) -> Result<Self::Value, A::Error>
    where
        A: de::ElementAccess<'v>,
    {
        let name = element.name().clone().into_owned();
        let attributes = iter::from_fn(|| match element.next_attribute::<XmlAttribute>() {
            Ok(Some(attr)) => Some(Ok(attr)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        })
        .collect::<Result<_, _>>()?;
        let mut children = element.children()?;

        let children = iter::from_fn(|| match children.next_element::<XmlChild>() {
            Ok(Some(child)) => Some(Ok(child)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        })
        .collect::<Result<_, _>>()?;

        Ok(XmlElement {
            name,
            attributes,
            children,
            preferred_prefix: None,
            enforce_prefix: crate::ser::IncludePrefix::Never,
        })
    }
}

impl<'de> crate::de::Deserialize<'de> for XmlElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: crate::de::Deserializer<'de>,
    {
        deserializer.deserialize_any(XmlElementVisitor::new())
    }
}

impl ser::SerializeAttributes for &mut XmlElement {
    type Ok = ();

    type Error = XmlValueSerializerError;

    fn serialize_attribute<A: SerializeAttribute>(
        &mut self,
        a: &A,
    ) -> Result<Self::Ok, Self::Error> {
        a.serialize_attribute(self)?;

        Ok(())
    }
}

impl ser::AttributeSerializer for &mut &mut XmlElement {
    type Ok = ();
    type Error = XmlValueSerializerError;

    type SerializeAttribute<'a>
        = XmlAttributeBuilder<'a>
    where
        Self: 'a;

    fn serialize_attribute(
        &mut self,
        name: &'_ ExpandedName<'_>,
    ) -> Result<Self::SerializeAttribute<'_>, Self::Error> {
        Ok(XmlAttributeBuilder {
            name: name.clone().into_owned(),
            write_to: &mut self.attributes,
            should_enforce: IncludePrefix::default(),
            preferred_prefix: None,
        })
    }

    fn serialize_none(&mut self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'s> ser::SerializeElementAttributes for &'s mut XmlElement {
    type ChildrenSerializeSeq = &'s mut XmlSeq<XmlChild>;

    fn serialize_children(self) -> Result<Self::ChildrenSerializeSeq, Self::Error> {
        Ok(&mut self.children)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'s> ser::SerializeElement for &'s mut XmlElement {
    type Ok = ();

    type Error = XmlValueSerializerError;

    type ChildrenSerializeSeq = &'s mut XmlSeq<XmlChild>;
    type SerializeElementAttributes = &'s mut XmlElement;

    fn include_prefix(&mut self, should_enforce: IncludePrefix) -> Result<Self::Ok, Self::Error> {
        self.enforce_prefix = should_enforce;
        Ok(())
    }

    fn preferred_prefix(
        &mut self,
        preferred_prefix: Option<crate::Prefix<'_>>,
    ) -> Result<Self::Ok, Self::Error> {
        self.preferred_prefix = preferred_prefix.map(Prefix::into_owned);
        Ok(())
    }

    fn serialize_children(self) -> Result<Self::ChildrenSerializeSeq, Self::Error> {
        Ok(&mut self.children)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_attributes(self) -> Result<Self::SerializeElementAttributes, Self::Error> {
        Ok(self)
    }
}

struct XmlElementAccess<'de, 'i> {
    element: &'de XmlElement,
    attribute_index: usize,
    write_attribute_index_to: Option<&'i mut usize>,
}

impl Drop for XmlElementAccess<'_, '_> {
    fn drop(&mut self) {
        if let Some(write_to) = self.write_attribute_index_to.as_mut() {
            **write_to = self.attribute_index;
        }
    }
}

impl<'de> AttributesAccess<'de> for XmlElementAccess<'de, '_> {
    type Error = XmlValueDeserializerError;

    type SubAccess<'a>
        = XmlElementAccess<'de, 'a>
    where
        Self: 'a;

    fn next_attribute<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        let Some(attribute) = self.element.attributes.get(self.attribute_index) else {
            return Ok(None);
        };
        let attribute = T::deserialize(attribute)?;
        self.attribute_index += 1;
        Ok(Some(attribute))
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        Ok(XmlElementAccess {
            attribute_index: self.attribute_index,
            element: self.element,
            write_attribute_index_to: Some(&mut self.attribute_index),
        })
    }
}

impl<'de> ElementAccess<'de> for XmlElementAccess<'de, '_> {
    type ChildrenAccess = XmlSeqAccess<'de, 'static, XmlChild>;
    type NamespaceContext<'a>
        = ()
    where
        Self: 'a;

    fn name(&self) -> ExpandedName<'_> {
        self.element.name.clone()
    }

    fn children(self) -> Result<Self::ChildrenAccess, Self::Error> {
        Ok(XmlSeqAccess {
            seq: &self.element.children,
            index: 0,
            write_index_to: None,
        })
    }

    fn namespace_context(&self) -> Self::NamespaceContext<'_> {}
}

impl<'de> Deserializer<'de> for &'de XmlElement {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_element(XmlElementAccess {
            element: self,
            attribute_index: 0,
            write_attribute_index_to: None,
        })
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

/// An XML attribute.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct XmlAttribute {
    /// The name of the attribute.
    pub name: ExpandedName<'static>,
    /// The value of the attribute.
    pub value: String,
}

impl XmlAttribute {
    /// Creates a new XML attribute.
    pub fn new<T: Into<ExpandedName<'static>>, U: Into<String>>(name: T, value: U) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

impl SerializeAttribute for XmlAttribute {
    fn serialize_attribute<S>(&self, mut serializer: S) -> Result<S::Ok, S::Error>
    where
        S: AttributeSerializer,
    {
        let attr = serializer.serialize_attribute(&self.name)?;

        attr.end(self.value.as_str())
    }
}

struct XmlAttributeVisitor<'v> {
    marker: ::core::marker::PhantomData<XmlAttribute>,
    lifetime: ::core::marker::PhantomData<&'v ()>,
}

impl XmlAttributeVisitor<'_> {
    fn new() -> Self {
        Self {
            marker: ::core::marker::PhantomData,
            lifetime: ::core::marker::PhantomData,
        }
    }
}

impl<'v> crate::de::Visitor<'v> for XmlAttributeVisitor<'v> {
    type Value = XmlAttribute;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an attribute")
    }

    fn visit_attribute<A>(self, attribute: A) -> Result<Self::Value, A::Error>
    where
        A: de::AttributeAccess<'v>,
    {
        Ok(XmlAttribute {
            name: attribute.name().clone().into_owned(),
            value: attribute.value().to_owned(),
        })
    }
}

impl<'a> de::AttributeAccess<'a> for &'a XmlAttribute {
    type Error = XmlValueDeserializerError;
    type NamespaceContext<'b>
        = ()
    where
        Self: 'b;

    fn name(&self) -> ExpandedName<'_> {
        self.name.clone()
    }

    fn value(&self) -> &str {
        &self.value
    }

    fn namespace_context(&self) -> Self::NamespaceContext<'_> {}
}

impl<'de> crate::de::Deserialize<'de> for XmlAttribute {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: crate::de::Deserializer<'de>,
    {
        deserializer.deserialize_any(XmlAttributeVisitor::new())
    }
}

/// Builder used when serializing to an [`XmlAttribute``].
pub struct XmlAttributeBuilder<'a> {
    name: ExpandedName<'static>,
    write_to: &'a mut Vec<XmlAttribute>,
    should_enforce: IncludePrefix,
    preferred_prefix: Option<Prefix<'static>>,
}

impl<'a> XmlAttributeBuilder<'a> {
    /// Creates a new [`XmlAttributeBuilder`].
    pub fn new(name: ExpandedName<'static>, write_to: &'a mut Vec<XmlAttribute>) -> Self {
        Self {
            name,
            write_to,
            should_enforce: IncludePrefix::default(),
            preferred_prefix: None,
        }
    }
}

impl SerializeAttributeAccess for XmlAttributeBuilder<'_> {
    type Ok = ();

    type Error = XmlValueSerializerError;

    fn include_prefix(&mut self, should_enforce: IncludePrefix) -> Result<Self::Ok, Self::Error> {
        self.should_enforce = should_enforce;
        Ok(())
    }

    fn preferred_prefix(
        &mut self,
        preferred_prefix: Option<crate::Prefix<'_>>,
    ) -> Result<Self::Ok, Self::Error> {
        self.preferred_prefix = preferred_prefix.map(|p| p.into_owned());
        Ok(())
    }

    fn end<S: AsRef<str>>(self, value: S) -> Result<Self::Ok, Self::Error> {
        self.write_to.push(XmlAttribute {
            name: self.name,
            value: value.as_ref().to_string(),
        });
        Ok(())
    }
}

impl<'de> crate::de::Deserializer<'de> for &'de XmlAttribute {
    type Error = XmlValueDeserializerError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_attribute(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_attribute(self)
    }
}

/// A sequence of XML elements.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct XmlSeq<T> {
    values: Vec<T>,
}

impl<T> IntoIterator for XmlSeq<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
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

impl<T> From<Vec<T>> for XmlSeq<T> {
    fn from(value: Vec<T>) -> Self {
        Self::from_vec(value)
    }
}

impl<T> XmlSeq<T> {
    /// Creates a new empty sequence.
    pub fn new() -> Self {
        Self::from_vec(Vec::new())
    }

    fn from_vec(values: Vec<T>) -> Self {
        Self { values }
    }

    /// Pushes a value onto the sequence.
    pub fn push(&mut self, value: T) {
        self.values.push(value);
    }
}

impl<T> Default for XmlSeq<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Serialize> Serialize for XmlSeq<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: crate::ser::Serializer,
    {
        let mut seq = serializer.serialize_seq()?;
        for item in self.values.iter() {
            seq.serialize_element(item)?;
        }

        seq.end()
    }
}

impl crate::ser::SerializeSeq for &mut XmlSeq<XmlValue> {
    type Ok = ();

    type Error = XmlValueSerializerError;

    fn serialize_element<V: Serialize>(&mut self, v: &V) -> Result<Self::Ok, Self::Error> {
        v.serialize(self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl crate::ser::SerializeSeq for &mut XmlSeq<XmlChild> {
    type Ok = ();

    type Error = XmlValueSerializerError;

    fn serialize_element<V: Serialize>(&mut self, v: &V) -> Result<Self::Ok, Self::Error> {
        v.serialize(self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

struct XmlSeqAccess<'de, 'i, T> {
    seq: &'de XmlSeq<T>,
    index: usize,
    write_index_to: Option<&'i mut usize>,
}

impl<'de, T> XmlSeqAccess<'de, '_, T> {
    pub fn new(seq: &'de XmlSeq<T>) -> Self {
        Self {
            seq,
            index: 0,
            write_index_to: None,
        }
    }
}

impl<T> Drop for XmlSeqAccess<'_, '_, T> {
    fn drop(&mut self) {
        if let Some(write_index_to) = self.write_index_to.as_mut() {
            **write_index_to = self.index;
        }
    }
}

// One would think that these impls for XmlSeqAccess that take XmlValue and XmlChild should be unified using a generic impl, but this does not appear to be possible due to an error mentioning limits in the borrow checker.
// I've fought the borrow checker for a long time and lost, so for now, these are separate impls.
// If you want to take a stab at unifying these, be my guest.
impl<'de> de::SeqAccess<'de> for XmlSeqAccess<'de, '_, XmlChild> {
    type Error = XmlValueDeserializerError;
    type SubAccess<'g>
        = XmlSeqAccess<'de, 'g, XmlChild>
    where
        Self: 'g;
    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        let Some(value) = self.seq.values.get(self.index) else {
            return Ok(None);
        };
        let value = T::deserialize(value)?;
        self.index += 1;
        Ok(Some(value))
    }

    fn next_element_seq<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        T::deserialize_seq(self).map(Some)
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        Ok(XmlSeqAccess {
            seq: self.seq,
            index: self.index,
            write_index_to: Some(&mut self.index),
        })
    }
}

impl<'de> de::SeqAccess<'de> for XmlSeqAccess<'de, '_, XmlValue> {
    type Error = XmlValueDeserializerError;
    type SubAccess<'g>
        = XmlSeqAccess<'de, 'g, XmlValue>
    where
        Self: 'g;
    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        let Some(value) = self.seq.values.get(self.index) else {
            return Ok(None);
        };
        let value = T::deserialize(value)?;
        self.index += 1;
        Ok(Some(value))
    }

    fn next_element_seq<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>,
    {
        T::deserialize_seq(self).map(Some)
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        Ok(XmlSeqAccess {
            seq: self.seq,
            index: self.index,
            write_index_to: Some(&mut self.index),
        })
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for XmlSeq<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: crate::de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(IteratorVisitor::<_, Self>::default())
    }
}

impl<'de> Deserializer<'de> for &mut XmlSeqAccess<'de, '_, XmlValue> {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl<'de> Deserializer<'de> for &mut XmlSeqAccess<'de, '_, XmlChild> {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl<'de> Deserializer<'de> for &'de XmlSeq<XmlValue> {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(XmlSeqAccess::new(self))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl<'de> Deserializer<'de> for &'de XmlSeq<XmlChild> {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(XmlSeqAccess::new(self))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
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

impl de::XmlProcessingInstruction for &XmlProcessingInstruction {
    type NamespaceContext<'a>
        = ()
    where
        Self: 'a;

    fn content(&self) -> &[u8] {
        self.content.as_slice()
    }

    fn target(&self) -> &[u8] {
        self.target.as_slice()
    }
    fn namespace_context(&self) -> Self::NamespaceContext<'_> {}
}

impl Serialize for XmlProcessingInstruction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: crate::ser::Serializer,
    {
        serializer.serialize_pi(&self.target, &self.content)
    }
}

struct XmlProcessingInstructionVisitor<'v> {
    marker: ::core::marker::PhantomData<XmlProcessingInstruction>,
    lifetime: ::core::marker::PhantomData<&'v ()>,
}

impl XmlProcessingInstructionVisitor<'_> {
    fn new() -> Self {
        Self {
            marker: ::core::marker::PhantomData,
            lifetime: ::core::marker::PhantomData,
        }
    }
}

impl<'v> crate::de::Visitor<'v> for XmlProcessingInstructionVisitor<'v> {
    type Value = XmlProcessingInstruction;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a comment")
    }

    fn visit_pi<E, V>(self, value: V) -> Result<Self::Value, E>
    where
        E: de::Error,
        V: de::XmlProcessingInstruction,
    {
        Ok(XmlProcessingInstruction {
            target: value.target().to_vec(),
            content: value.content().to_vec(),
        })
    }
}

impl<'de> Deserialize<'de> for XmlProcessingInstruction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(XmlProcessingInstructionVisitor::new())
    }
}

impl<'de> Deserializer<'de> for &'de XmlProcessingInstruction {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_pi(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
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

impl de::XmlDeclaration for &XmlDecl {
    type NamespaceContext<'a>
        = ()
    where
        Self: 'a;

    fn version(&self) -> &[u8] {
        self.version.as_bytes()
    }

    fn encoding(&self) -> Option<&[u8]> {
        self.encoding.as_deref().map(|e| e.as_bytes())
    }

    fn standalone(&self) -> Option<&[u8]> {
        self.standalone.as_deref().map(|s| s.as_bytes())
    }

    fn namespace_context(&self) -> Self::NamespaceContext<'_> {}
}

impl Serialize for XmlDecl {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_decl(
            self.version.deref(),
            self.encoding.as_deref(),
            self.standalone.as_deref(),
        )
    }
}

struct XmlDeclVisitor<'v> {
    marker: ::core::marker::PhantomData<XmlDecl>,
    lifetime: ::core::marker::PhantomData<&'v ()>,
}

impl XmlDeclVisitor<'_> {
    fn new() -> Self {
        Self {
            marker: ::core::marker::PhantomData,
            lifetime: ::core::marker::PhantomData,
        }
    }
}

impl<'v> crate::de::Visitor<'v> for XmlDeclVisitor<'v> {
    type Value = XmlDecl;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a declaration")
    }

    fn visit_decl<E, V>(self, declaration: V) -> Result<Self::Value, E>
    where
        E: de::Error,
        V: de::XmlDeclaration,
    {
        Ok(XmlDecl {
            version: String::from_utf8_lossy(declaration.version()).to_string(),
            encoding: declaration
                .encoding()
                .map(|e| String::from_utf8_lossy(e).to_string()),
            standalone: declaration
                .standalone()
                .map(|e| String::from_utf8_lossy(e).to_string()),
        })
    }
}

impl<'de> Deserialize<'de> for XmlDecl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(XmlDeclVisitor::new())
    }
}

impl<'de> Deserializer<'de> for &'de XmlDecl {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_decl(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
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

impl de::XmlComment for &XmlComment {
    type NamespaceContext<'a>
        = ()
    where
        Self: 'a;
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    fn namespace_context(&self) -> Self::NamespaceContext<'_> {}
}

impl Serialize for XmlComment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_comment(&self.0)
    }
}

struct XmlCommentVisitor<'v> {
    marker: ::core::marker::PhantomData<XmlComment>,
    lifetime: ::core::marker::PhantomData<&'v ()>,
}

impl XmlCommentVisitor<'_> {
    pub fn new() -> Self {
        Self {
            marker: ::core::marker::PhantomData,
            lifetime: ::core::marker::PhantomData,
        }
    }
}

impl<'v> crate::de::Visitor<'v> for XmlCommentVisitor<'v> {
    type Value = XmlComment;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a comment")
    }

    fn visit_comment<E, V>(self, comment: V) -> Result<Self::Value, E>
    where
        E: de::Error,
        V: de::XmlComment,
    {
        Ok(XmlComment(comment.as_bytes().to_vec()))
    }
}

impl<'de> Deserialize<'de> for XmlComment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(XmlCommentVisitor::new())
    }
}

impl<'de> Deserializer<'de> for &'de XmlComment {
    type Error = XmlValueDeserializerError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_comment(self)
    }
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
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

impl de::XmlDoctype for &XmlDoctype {
    type NamespaceContext<'a>
        = ()
    where
        Self: 'a;

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn namespace_context(&self) -> Self::NamespaceContext<'_> {}
}

impl Serialize for XmlDoctype {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_doctype(&self.0)
    }
}

struct XmlDoctypeVisitor<'v> {
    marker: ::core::marker::PhantomData<XmlDoctype>,
    lifetime: ::core::marker::PhantomData<&'v ()>,
}

impl XmlDoctypeVisitor<'_> {
    pub fn new() -> Self {
        Self {
            marker: ::core::marker::PhantomData,
            lifetime: ::core::marker::PhantomData,
        }
    }
}

impl<'v> crate::de::Visitor<'v> for XmlDoctypeVisitor<'v> {
    type Value = XmlDoctype;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a comment")
    }

    fn visit_doctype<E, V>(self, value: V) -> Result<Self::Value, E>
    where
        E: de::Error,
        V: de::XmlDoctype,
    {
        Ok(XmlDoctype(value.as_bytes().to_vec()))
    }
}

impl<'de> Deserialize<'de> for XmlDoctype {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(XmlDoctypeVisitor::new())
    }
}

impl<'de> Deserializer<'de> for &'de XmlDoctype {
    type Error = XmlValueDeserializerError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_doctype(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

/// Error type for serializing XML values.
#[derive(Debug, thiserror::Error)]
pub enum XmlValueSerializerError {
    /// Error for when a custom error occurs during serialization.
    #[error("Custom error: {0}")]
    Custom(String),
    /// Error for when an invalid child deserialization occurs.
    #[error("Invalid child deserialization")]
    InvalidChildDeserialization,
}

impl ser::Error for XmlValueSerializerError {
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
        actual: Box<ExpandedName<'static>>,
        /// The expected name.
        expected: Box<ExpandedName<'static>>,
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
            actual: Box::new(name.clone().into_owned()),
            expected: Box::new(expected.clone().into_owned()),
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
