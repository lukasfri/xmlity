#![warn(missing_docs)]
//! # XMLity - Powerful XML (de)serialization for Rust
//!
//! XMLity is a (de)serialization library for XML in Rust designed to allow for practically any kind of XML structure to be (de)serialized.
//!
//! Currently, XMLity does not currently optimize for ergonomics, which means that it is quite verbose to use. However, it is designed to be easy to use and extend.
//!
//! The most important types in XMLity are:
//! - [`Serialize`], [`SerializeAttribute`] and [`Deserialize`] which lets you define how to (de)serialize types,
//! - [`Serializer`] and [`Deserializer`] which lets you define readers and writers for XML documents.
//! - [`SerializationGroup`] and [`DeserializationGroup`] which lets you define how to (de)serialize groups of types that can be extended upon in other elements/groups recursively.
//! - [`XmlValue`] and its variants which allow for generic deserialization of XML documents, similar to [`serde_json::Value`].
//!
//! The library includes derive macros for [`Serialize`], [`SerializeAttribute`], [`Deserialize`], [`SerializationGroup`] and [`DeserializationGroup`] which can be enabled with the `derive` feature. The macros can be used to create nearly any kind of XML structure you want. If there is something it cannot do, please open an issue or a pull request.
//!
//! The macro [`xml`] can be used to create [`XmlValues`](`XmlValue`) in a more ergonomic way. It is also possible to create [`XmlValues`](`XmlValue`) manually, but it is quite verbose.
use core::{fmt, str};
use fmt::Display;
use std::{borrow::Cow, str::FromStr};

pub mod de;
pub use de::{DeserializationGroup, Deserialize, DeserializeOwned, Deserializer};
pub mod ser;
pub use ser::{AttributeSerializer, SerializationGroup, Serialize, SerializeAttribute, Serializer};
mod macros;
pub mod types;
pub mod value;
pub use value::XmlValue;
mod noop;
pub use noop::NoopDeSerializer;

#[cfg(feature = "derive")]
extern crate xmlity_derive;

#[cfg(feature = "derive")]
pub use xmlity_derive::{
    DeserializationGroup, Deserialize, SerializationGroup, Serialize, SerializeAttribute,
};

// Reference: https://www.w3.org/TR/xml/#sec-common-syn
// [4]   	NameStartChar	   ::=   	":" | [A-Z] | "_" | [a-z] | [#xC0-#xD6] | [#xD8-#xF6] | [#xF8-#x2FF] | [#x370-#x37D] | [#x37F-#x1FFF] | [#x200C-#x200D] | [#x2070-#x218F] | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
// [4a]   	NameChar	   ::=   	NameStartChar | "-" | "." | [0-9] | #xB7 | [#x0300-#x036F] | [#x203F-#x2040]
// [5]   	Name	   ::=   	NameStartChar (NameChar)*
// [6]   	Names	   ::=   	Name (#x20 Name)*
// [7]   	Nmtoken	   ::=   	(NameChar)+
// [8]   	Nmtokens	   ::=   	Nmtoken (#x20 Nmtoken)*
mod name_tokens {
    const fn is_name_start_char(c: char) -> bool {
        matches!(
            //Deliberately excluding : as we handle it separately
            c, 'A'..='Z' | '_' | 'a'..='z' | '\u{00C0}'..='\u{00D6}' | '\u{00D8}'..='\u{00F6}' | '\u{00F8}'..='\u{02FF}' | '\u{0370}'..='\u{037D}' | '\u{037F}'..='\u{1FFF}' | '\u{200C}'..='\u{200D}' | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}' | '\u{3001}'..='\u{D7FF}' | '\u{F900}'..='\u{FDCF}' | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..='\u{EFFFF}'
        )
    }
    const fn is_name_char(c: char) -> bool {
        is_name_start_char(c)
            || matches!(c, '-' | '.' | '0'..='9' | '\u{00B7}' | '\u{0300}'..='\u{036F}' | '\u{203F}'..='\u{2040}')
    }

    /// An error that occurs when a name is invalid.
    #[derive(Debug, Clone, thiserror::Error, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[non_exhaustive]
    pub enum InvalidXmlNameError {
        /// The name starts with an invalid character. The first character of an XML name is special and excludes some other characters incl "-" which is allowed in the middle of the name.
        #[error("Invalid start character")]
        InvalidStartChar,
        /// The name contains an invalid character.
        #[error("Invalid character at index {index}")]
        InvalidChar {
            /// The index of the invalid character.
            index: usize,
            /// The invalid character.
            character: char,
        },
        /// The name is empty.
        #[error("Empty name")]
        Empty,
    }

    pub fn is_valid_name(name: &str) -> Result<(), InvalidXmlNameError> {
        let mut chars = name.chars();
        if let Some(c) = chars.next() {
            if !is_name_start_char(c) {
                return Err(InvalidXmlNameError::InvalidStartChar);
            }
        } else {
            return Err(InvalidXmlNameError::Empty);
        }
        for (index, character) in chars.enumerate().map(|(i, c)| (i + 1, c)) {
            if !is_name_char(character) {
                return Err(InvalidXmlNameError::InvalidChar { index, character });
            }
        }

        Ok(())
    }
}

pub use name_tokens::InvalidXmlNameError;

/// # XML Expanded Name
/// An [`ExpandedName`] is a [`LocalName`] together with its associated [`XmlNamespace`]. This can convert to and from a [`QName`] with a [`Prefix`] mapping.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExpandedName<'a> {
    local_name: LocalName<'a>,
    namespace: Option<XmlNamespace<'a>>,
}

impl<'a> ExpandedName<'a> {
    /// Creates a new [`ExpandedName`].
    pub fn new(local_name: LocalName<'a>, namespace: Option<XmlNamespace<'a>>) -> Self {
        Self {
            local_name,
            namespace,
        }
    }

    /// Converts this [`ExpandedName`] into its parts.
    pub fn into_parts(self) -> (LocalName<'a>, Option<XmlNamespace<'a>>) {
        (self.local_name, self.namespace)
    }

    /// Converts this [`ExpandedName`] into an owned version.
    pub fn into_owned(self) -> ExpandedName<'static> {
        ExpandedName::new(
            self.local_name.into_owned(),
            self.namespace.map(|n| n.into_owned()),
        )
    }

    /// Returns this [`ExpandedName`] as a reference.
    pub fn as_ref(&self) -> ExpandedName<'_> {
        ExpandedName::new(
            self.local_name.as_ref(),
            self.namespace.as_ref().map(|n| n.as_ref()),
        )
    }

    /// Returns the local name of this [`ExpandedName`].
    pub fn local_name(&self) -> &LocalName<'_> {
        &self.local_name
    }

    /// Returns the namespace of this [`ExpandedName`].
    pub fn namespace(&self) -> Option<&XmlNamespace<'_>> {
        self.namespace.as_ref()
    }

    /// Converts this [`ExpandedName`] into a [`QName`] name using the given [`Prefix`].
    pub fn to_q_name(self, resolved_prefix: Option<Prefix<'a>>) -> QName<'a> {
        QName::new(resolved_prefix, self.local_name.clone())
    }
}

impl Display for ExpandedName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.local_name.fmt(f)
    }
}

/// # XML Qualified Name
/// A [`QName`] is a [`LocalName`] together with a namespace [`Prefix`], indicating it belongs to a specific declared [`XmlNamespace`].
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct QName<'a> {
    prefix: Option<Prefix<'a>>,
    local_name: LocalName<'a>,
}

/// An error that can occur when parsing a [`QName`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum QNameParseError {
    /// The [`Prefix`] is invalid.
    #[error("Invalid prefix: {0}")]
    InvalidPrefix(#[from] PrefixParseError),
    /// The [`LocalName`] is invalid.
    #[error("Invalid local name: {0}")]
    InvalidLocalName(#[from] LocalNameParseError),
}

impl<'a> QName<'a> {
    /// Creates a new [`QName`].
    pub fn new<P: Into<Option<Prefix<'a>>>, L: Into<LocalName<'a>>>(
        prefix: P,
        local_name: L,
    ) -> Self {
        QName {
            prefix: prefix.into(),
            local_name: local_name.into(),
        }
    }

    /// Converts this [`QName`] into its parts.
    pub fn into_parts(self) -> (Option<Prefix<'a>>, LocalName<'a>) {
        (self.prefix, self.local_name)
    }

    /// Converts this [`QName`] into being owned.
    pub fn into_owned(self) -> QName<'static> {
        QName {
            prefix: self.prefix.map(|prefix| prefix.into_owned()),
            local_name: self.local_name.into_owned(),
        }
    }

    /// Returns this [`QName`] as a reference.
    pub fn as_ref(&self) -> QName<'_> {
        QName {
            prefix: self.prefix.as_ref().map(|prefix| prefix.as_ref()),
            local_name: self.local_name.as_ref(),
        }
    }

    /// Returns the [`Prefix`] of this [`QName`].
    pub fn prefix(&self) -> Option<&Prefix<'a>> {
        self.prefix.as_ref()
    }

    /// Returns the [`LocalName`] of this [`QName`].
    pub fn local_name(&self) -> &LocalName<'a> {
        &self.local_name
    }
}

impl Display for QName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(prefix) = &self.prefix.as_ref().filter(|prefix| !prefix.is_default()) {
            write!(f, "{}:{}", prefix, self.local_name)
        } else {
            write!(f, "{}", self.local_name)
        }
    }
}
impl FromStr for QName<'_> {
    type Err = QNameParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (prefix, local_name) = s.split_once(':').unwrap_or(("", s));

        let prefix = Prefix::from_str(prefix)?;
        let local_name = LocalName::from_str(local_name)?;

        Ok(QName::new(prefix, local_name))
    }
}

impl<'a> From<QName<'a>> for Option<Prefix<'a>> {
    fn from(value: QName<'a>) -> Self {
        value.prefix
    }
}

impl<'a> From<QName<'a>> for LocalName<'a> {
    fn from(value: QName<'a>) -> Self {
        value.local_name
    }
}

/// # XML Namespace
/// A namespace URI, to which [`LocalNames`](`LocalName`) are scoped under.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct XmlNamespace<'a>(Cow<'a, str>);

/// An error that can occur when parsing a [`XmlNamespace`].
#[derive(Debug, thiserror::Error)]
pub enum XmlNamespaceParseError {}

impl<'a> XmlNamespace<'a> {
    /// Creates a new [`XmlNamespace`] from a string.
    pub fn new<T: Into<Cow<'a, str>>>(value: T) -> Result<Self, XmlNamespaceParseError> {
        Ok(Self(value.into()))
    }

    /// Creates a new [`XmlNamespace`] from a string without validating it, but it works in a const context.
    ///
    /// # Safety
    /// This function does not validate the input value due to const context limitations involving the validation function. While this cannot create a memory safety issue, it can create a logical one.
    pub const fn new_dangerous(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }

    /// Converts this [`XmlNamespace`] into an owned version.
    pub fn into_owned(self) -> XmlNamespace<'static> {
        XmlNamespace(Cow::Owned(self.0.into_owned()))
    }

    /// Returns this [`XmlNamespace`] as a reference.
    pub fn as_ref(&self) -> XmlNamespace<'_> {
        XmlNamespace(Cow::Borrowed(&self.0))
    }

    /// Returns this [`XmlNamespace`] as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The namespace for XML namespace declarations.
    pub const XMLNS: XmlNamespace<'static> =
        XmlNamespace::new_dangerous("http://www.w3.org/2000/xmlns/");
    /// The namespace for built-in XML attributes.
    pub const XML: XmlNamespace<'static> =
        XmlNamespace::new_dangerous("http://www.w3.org/XML/1998/namespace");
    /// The namespace for XHTML.
    pub const XHTML: XmlNamespace<'static> =
        XmlNamespace::new_dangerous("http://www.w3.org/1999/xhtml");
    /// The namespace for XML Schema Instance.
    pub const XSI: XmlNamespace<'static> =
        XmlNamespace::new_dangerous("http://www.w3.org/2001/XMLSchema-instance");
}

impl FromStr for XmlNamespace<'_> {
    type Err = XmlNamespaceParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value.to_owned())
    }
}

impl Display for XmlNamespace<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for XmlNamespace<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_text(&self.0)
    }
}

impl<'de> Deserialize<'de> for XmlNamespace<'_> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(types::string::FromStrVisitor::default())
    }
}

/// # XML Prefix
/// A namespace [`Prefix`] used to map a [`LocalName`] to a [`XmlNamespace`] within an XML document.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Prefix<'a>(Cow<'a, str>);

/// An error that can occur when parsing a [`Prefix`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PrefixParseError {
    /// The [`Prefix`] is not a valid XML name.
    #[error("Prefix has an invalid XML name: {0}")]
    InvalidXmlName(#[from] name_tokens::InvalidXmlNameError),
}

impl<'a> Prefix<'a> {
    /// Creates a new [`Prefix`] from a string.
    pub fn new<T: Into<Cow<'a, str>>>(value: T) -> Result<Self, PrefixParseError> {
        let value = value.into();

        name_tokens::is_valid_name(&value)?;

        Ok(Self(value))
    }

    /// Creates a new [`Prefix`] from a string without validating it, but it works in a const context.
    ///
    /// # Safety
    /// This function does not validate the input value due to const context limitations involving the validation function. While this cannot create a memory safety issue, it can create a logical one.
    pub const fn new_dangerous(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }

    /// Converts this [`Prefix`] into an owned version.
    pub fn into_owned(self) -> Prefix<'static> {
        Prefix(Cow::Owned(self.0.into_owned()))
    }

    /// Returns this [`Prefix`] as a reference.
    pub fn as_ref(&self) -> Prefix<'_> {
        Prefix(Cow::Borrowed(&self.0))
    }

    /// Returns this [`Prefix`] as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns this [`Prefix`] as a [`QName`] with the `xmlns` prefix. This is useful for serializing namespaces.
    pub fn xmlns(&'a self) -> QName<'a> {
        QName::new(
            Prefix::new("xmlns").expect("xmlns is a valid prefix"),
            LocalName::from(self.clone()),
        )
    }

    /// Returns `true` if this [`Prefix`] is the default prefix.
    pub fn is_default(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> From<Prefix<'a>> for LocalName<'a> {
    fn from(value: Prefix<'a>) -> Self {
        LocalName(value.0)
    }
}

impl<'a> From<Option<Prefix<'a>>> for Prefix<'a> {
    fn from(value: Option<Prefix<'a>>) -> Self {
        value.unwrap_or_default()
    }
}

impl FromStr for Prefix<'_> {
    type Err = PrefixParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value.to_owned())
    }
}

impl Display for Prefix<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for Prefix<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_text(&self.0)
    }
}

impl<'de> Deserialize<'de> for Prefix<'_> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(types::string::FromStrVisitor::default())
    }
}

/// # XML Local Name
/// A local name of an XML element or attribute within a [`XmlNamespace`].
///
/// Together with a [`XmlNamespace`], it forms an [`ExpandedName`].
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocalName<'a>(Cow<'a, str>);

/// An error that can occur when parsing a [`LocalName`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LocalNameParseError {
    /// The [`LocalName`] is not a valid XML name.
    #[error("Local name has an invalid XML name: {0}")]
    InvalidXmlName(#[from] name_tokens::InvalidXmlNameError),
}

impl<'a> LocalName<'a> {
    /// Creates a new [`LocalName`] from a string.
    pub fn new<T: Into<Cow<'a, str>>>(value: T) -> Result<Self, LocalNameParseError> {
        let value = value.into();

        name_tokens::is_valid_name(&value)?;

        Ok(Self(value))
    }

    /// Creates a new [`LocalName`] from a string without validating it, but it works in a const context.
    ///
    /// # Safety
    /// This function does not validate the input value due to const context limitations involving the validation function. While this cannot create a memory safety issue, it can create a logical one.
    pub const fn new_dangerous(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }

    /// Converts this [`LocalName`] into an owned version.
    pub fn into_owned(self) -> LocalName<'static> {
        LocalName(Cow::Owned(self.0.into_owned()))
    }

    /// Returns this [`LocalName`] as a reference.
    pub fn as_ref(&self) -> LocalName<'_> {
        LocalName(Cow::Borrowed(&self.0))
    }
}

impl FromStr for LocalName<'_> {
    type Err = LocalNameParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_owned())
    }
}

impl Display for LocalName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for LocalName<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_text(&self.0)
    }
}

impl<'de> Deserialize<'de> for LocalName<'_> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(types::string::FromStrVisitor::default())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use std::str::FromStr;

    #[rstest]
    #[case::basic("prefix")]
    fn test_prefix(#[case] prefix_text: &str) {
        let prefix = Prefix::from_str(prefix_text).unwrap();
        assert_eq!(prefix.to_string(), prefix_text);
        assert_eq!(prefix.into_owned().to_string(), prefix_text);
    }

    #[rstest]
    #[case::empty("", PrefixParseError::InvalidXmlName(InvalidXmlNameError::Empty))]
    #[case::space("invalid prefix", PrefixParseError::InvalidXmlName(InvalidXmlNameError::InvalidChar { index: 7, character: ' ' }))]
    fn invalid_prefix_invalid_characters(
        #[case] prefix: &str,
        #[case] expected_error: PrefixParseError,
    ) {
        let error = Prefix::from_str(prefix).unwrap_err();
        assert_eq!(error, expected_error);
    }

    #[rstest]
    #[case::basic("localName")]
    fn test_local_name(#[case] local_name_text: &str) {
        let local_name = LocalName::from_str(local_name_text).unwrap();
        assert_eq!(local_name.to_string(), local_name_text);
        assert_eq!(local_name.into_owned().to_string(), local_name_text);
    }

    #[rstest]
    #[case::empty("", LocalNameParseError::InvalidXmlName(InvalidXmlNameError::Empty))]
    #[case::space("invalid localName", LocalNameParseError::InvalidXmlName(InvalidXmlNameError::InvalidChar { index: 7, character: ' ' }))]
    fn invalid_local_name_invalid_characters(
        #[case] local_name: &str,
        #[case] expected_error: LocalNameParseError,
    ) {
        let error = LocalName::from_str(local_name).unwrap_err();
        assert_eq!(error, expected_error);
    }

    #[rstest]
    #[case::basic("localName", None)]
    #[case::with_namespace("localName", Some(XmlNamespace::new("http://example.com").unwrap()))]
    fn test_expanded_name(#[case] local_name_text: &str, #[case] namespace: Option<XmlNamespace>) {
        let local_name = LocalName::from_str(local_name_text).unwrap();
        let expanded_name = ExpandedName::new(local_name.clone(), namespace.clone());
        assert_eq!(expanded_name.local_name(), &local_name);
        assert_eq!(expanded_name.namespace(), namespace.as_ref());
        assert_eq!(expanded_name.to_string(), local_name_text);
        assert_eq!(expanded_name.into_owned().to_string(), local_name_text);
    }

    #[rstest]
    #[case::basic("prefix:localName")]
    fn test_qname(#[case] qname_text: &str) {
        let qname = QName::from_str(qname_text).unwrap();
        assert_eq!(qname.to_string(), qname_text);
        assert_eq!(qname.into_owned().to_string(), qname_text);
    }

    #[rstest]
    #[case::invalid_local_name("prefix:invalid localName", QNameParseError::InvalidLocalName(LocalNameParseError::InvalidXmlName(InvalidXmlNameError::InvalidChar { index: 7, character: ' ' })))]
    fn invalid_qname_invalid_characters(
        #[case] qname: &str,
        #[case] expected_error: QNameParseError,
    ) {
        let error = QName::from_str(qname).unwrap_err();
        assert_eq!(error, expected_error);
    }
}
