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
use std::{borrow::Borrow, ops::Deref, str::FromStr};

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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExpandedName<'a> {
    local_name: &'a LocalName,
    namespace: Option<&'a XmlNamespace>,
}

impl<'a> ExpandedName<'a> {
    /// Creates a new [`ExpandedName`].
    pub fn new(local_name: &'a LocalName, namespace: Option<&'a XmlNamespace>) -> Self {
        Self {
            local_name,
            namespace,
        }
    }

    /// Converts this [`ExpandedName`] into an owned version.
    pub fn into_owned(self) -> ExpandedNameBuf {
        ExpandedNameBuf::new(
            self.local_name.to_owned(),
            self.namespace.map(|n| n.to_owned()),
        )
    }

    /// Converts this [`ExpandedName`] into its parts.
    pub fn into_parts(self) -> (&'a LocalName, Option<&'a XmlNamespace>) {
        (self.local_name, self.namespace)
    }

    /// Returns the local name of this [`ExpandedName`].
    pub fn local_name(&self) -> &'a LocalName {
        self.local_name
    }

    /// Returns the namespace of this [`ExpandedName`].
    pub fn namespace(&self) -> &Option<&'a XmlNamespace> {
        &self.namespace
    }

    /// Converts this [`ExpandedName`] into a [`QName`] name using the given [`Prefix`].
    pub fn to_q_name(self, resolved_prefix: Option<&'a Prefix>) -> QName<'a> {
        QName::new(resolved_prefix, self.local_name)
    }
}

impl Display for ExpandedName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(namespace) = &self.namespace {
            write!(f, "{{{}}}{}", namespace, self.local_name)
        } else {
            write!(f, "{}", self.local_name)
        }
    }
}

/// An owned version of [`ExpandedName`].
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExpandedNameBuf {
    local_name: LocalNameBuf,
    namespace: Option<XmlNamespaceBuf>,
}

impl ExpandedNameBuf {
    /// Creates a new [`ExpandedNameBuf`].
    pub fn new(local_name: LocalNameBuf, namespace: Option<XmlNamespaceBuf>) -> Self {
        Self {
            local_name,
            namespace,
        }
    }

    /// Returns a reference to this [`ExpandedNameBuf`] as an [`ExpandedName`].
    pub fn as_ref(&self) -> ExpandedName<'_> {
        ExpandedName::new(
            self.local_name.borrow(),
            self.namespace.as_ref().map(|n| n.borrow()),
        )
    }

    /// Converts this [`ExpandedNameBuf`] into its parts.
    pub fn into_parts(self) -> (LocalNameBuf, Option<XmlNamespaceBuf>) {
        (self.local_name, self.namespace)
    }

    /// Returns the [`XmlNamespace`] of this [`ExpandedNameBuf`].
    pub fn namespace(&self) -> Option<&XmlNamespace> {
        self.namespace.as_deref()
    }

    /// Returns the [`LocalName`] of this [`ExpandedNameBuf`].
    pub fn local_name(&self) -> &LocalName {
        &self.local_name
    }
}

impl From<ExpandedName<'_>> for ExpandedNameBuf {
    fn from(value: ExpandedName<'_>) -> Self {
        value.into_owned()
    }
}

impl Display for ExpandedNameBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

/// # XML Qualified Name
/// A [`QName`] is a [`LocalName`] together with a namespace [`Prefix`], indicating it belongs to a specific declared [`XmlNamespace`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct QName<'a> {
    prefix: Option<&'a Prefix>,
    local_name: &'a LocalName,
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
    pub fn new(prefix: Option<&'a Prefix>, local_name: &'a LocalName) -> Self {
        QName { prefix, local_name }
    }

    /// Converts this [`QName`] into its parts.
    pub fn into_parts(self) -> (Option<&'a Prefix>, &'a LocalName) {
        (self.prefix, self.local_name)
    }

    /// Returns the [`Prefix`] of this [`QName`].
    pub fn prefix(&self) -> &Option<&'a Prefix> {
        &self.prefix
    }

    /// Returns the [`LocalName`] of this [`QName`].
    pub fn local_name(&self) -> &LocalName {
        self.local_name
    }

    /// Converts this [`QName`] into an owned version.
    pub fn into_owned(self) -> QNameBuf {
        QNameBuf::new(
            self.prefix.map(|p| p.to_owned()),
            self.local_name.to_owned(),
        )
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

/// An owned version of [`QName`].
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct QNameBuf {
    prefix: Option<PrefixBuf>,
    local_name: LocalNameBuf,
}

impl QNameBuf {
    /// Creates a new [`QNameBuf`].
    pub fn new(prefix: Option<PrefixBuf>, local_name: LocalNameBuf) -> Self {
        Self { prefix, local_name }
    }

    /// Returns a reference to this [`QNameBuf`] as a [`QName`].
    pub fn as_ref(&self) -> QName<'_> {
        QName::new(
            self.prefix.as_ref().map(|p| p.borrow()),
            self.local_name.borrow(),
        )
    }
}

impl FromStr for QNameBuf {
    type Err = QNameParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (prefix, local_name) = s.split_once(':').unwrap_or(("", s));

        let prefix = if prefix.is_empty() {
            None
        } else {
            Some(PrefixBuf::from_str(prefix)?)
        };
        let local_name = LocalNameBuf::from_str(local_name)?;

        Ok(QNameBuf::new(prefix, local_name))
    }
}

impl Display for QNameBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl From<QName<'_>> for QNameBuf {
    fn from(value: QName<'_>) -> Self {
        value.into_owned()
    }
}

impl PartialEq<QName<'_>> for QNameBuf {
    fn eq(&self, other: &QName<'_>) -> bool {
        self.as_ref() == *other
    }
}

impl PartialEq<QNameBuf> for QName<'_> {
    fn eq(&self, other: &QNameBuf) -> bool {
        *self == other.as_ref()
    }
}

/// # XML Namespace
/// A namespace URI, to which [`LocalNames`](`LocalName`) are scoped under.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct XmlNamespace(str);

/// An error that can occur when parsing a [`XmlNamespace`].
#[derive(Debug, thiserror::Error)]
pub enum XmlNamespaceParseError {}

impl XmlNamespace {
    /// Creates a new [`XmlNamespace`] from a string slice without validating it.
    ///
    /// # Safety
    /// The caller must ensure that the value is a valid URI.
    pub const unsafe fn new_unchecked(value: &str) -> &Self {
        // SAFETY: The caller must ensure that the value is a valid URI.
        unsafe { &*(value as *const str as *const XmlNamespace) }
    }

    /// Creates a new [`XmlNamespace`] from a string.
    pub fn new(value: &str) -> Result<&Self, XmlNamespaceParseError> {
        //TODO: Validate URI
        // SAFETY: The value has been validated.
        Ok(unsafe { Self::new_unchecked(value) })
    }

    /// Returns this [`XmlNamespace`] as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The namespace for XML namespace declarations.
    pub const XMLNS: &'static XmlNamespace =
        //SAFETY: Hardcoded valid URI.
        unsafe { XmlNamespace::new_unchecked("http://www.w3.org/2000/xmlns/") };
    /// The namespace for built-in XML attributes.
    pub const XML: &'static XmlNamespace =
        //SAFETY: Hardcoded valid URI.
        unsafe { XmlNamespace::new_unchecked("http://www.w3.org/XML/1998/namespace") };
    /// The namespace for XHTML.
    pub const XHTML: &'static XmlNamespace =
        //SAFETY: Hardcoded valid URI.
        unsafe { XmlNamespace::new_unchecked("http://www.w3.org/1999/xhtml") };
    /// The namespace for XML Schema.
    pub const XS: &'static XmlNamespace =
        //SAFETY: Hardcoded valid URI.
        unsafe { XmlNamespace::new_unchecked("http://www.w3.org/2001/XMLSchema") };
    /// The namespace for XML Schema Instance.
    pub const XSI: &'static XmlNamespace =
        //SAFETY: Hardcoded valid URI.
        unsafe { XmlNamespace::new_unchecked("http://www.w3.org/2001/XMLSchema-instance") };
}

impl Display for XmlNamespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for XmlNamespace {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_text(&self.0)
    }
}

/// An owned version of [`XmlNamespace`].
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct XmlNamespaceBuf(String);

impl XmlNamespaceBuf {
    /// Creates a new [`XmlNamespaceBuf`] from a string without validating it.
    ///
    /// # Safety
    /// The caller must ensure that the value is a valid URI.
    pub const unsafe fn new_unchecked(value: String) -> Self {
        Self(value)
    }

    /// Creates a new [`XmlNamespaceBuf`] from a string.
    pub fn new(value: String) -> Result<Self, XmlNamespaceParseError> {
        // Validate the value
        XmlNamespace::new(&value)?;

        // SAFETY: The value has been validated.
        Ok(unsafe { Self::new_unchecked(value) })
    }
}

impl Borrow<XmlNamespace> for XmlNamespaceBuf {
    fn borrow(&self) -> &XmlNamespace {
        // SAFETY: All XmlNamespaceBufs are valid XmlNamespaces.
        unsafe { XmlNamespace::new_unchecked(&self.0) }
    }
}

impl Deref for XmlNamespaceBuf {
    type Target = XmlNamespace;
    fn deref(&self) -> &Self::Target {
        self.borrow()
    }
}

impl ToOwned for XmlNamespace {
    type Owned = XmlNamespaceBuf;
    fn to_owned(&self) -> Self::Owned {
        XmlNamespaceBuf(self.0.to_owned())
    }
}

impl FromStr for XmlNamespaceBuf {
    type Err = XmlNamespaceParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        XmlNamespace::new(value).map(ToOwned::to_owned)
    }
}

impl<'de> Deserialize<'de> for XmlNamespaceBuf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(types::string::FromStrVisitor::default())
    }
}

impl Display for XmlNamespaceBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl PartialEq<XmlNamespace> for XmlNamespaceBuf {
    fn eq(&self, other: &XmlNamespace) -> bool {
        self.deref() == other
    }
}

impl PartialEq<XmlNamespaceBuf> for XmlNamespace {
    fn eq(&self, other: &XmlNamespaceBuf) -> bool {
        self == other.deref()
    }
}

/// # XML Prefix
/// A namespace [`Prefix`] used to map a [`LocalName`] to a [`XmlNamespace`] within an XML document.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Prefix(str);

/// An error that can occur when parsing a [`Prefix`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PrefixParseError {
    /// The [`Prefix`] is not a valid XML name.
    #[error("Prefix has an invalid XML name: {0}")]
    InvalidXmlName(#[from] name_tokens::InvalidXmlNameError),
}

impl Prefix {
    /// Creates a new [`Prefix`] from a string without validating it.
    ///
    /// # Safety
    /// The caller must ensure that the value is a valid XML name.
    pub const unsafe fn new_unchecked(value: &str) -> &Self {
        // SAFETY: The caller must ensure that the value is a valid XML name.
        unsafe { &*(value as *const str as *const Prefix) }
    }

    /// Creates a new [`Prefix`] from a string.
    pub fn new(value: &str) -> Result<&Self, PrefixParseError> {
        name_tokens::is_valid_name(value)?;

        // SAFETY: The value has been validated.
        Ok(unsafe { Self::new_unchecked(value) })
    }

    /// Converts this [`Prefix`] into [`PrefixBuf`].
    pub fn into_owned(&self) -> PrefixBuf {
        // SAFETY: All Prefixes are valid PrefixBufs.
        unsafe { PrefixBuf::new_unchecked(self.0.to_owned()) }
    }

    /// Returns this [`Prefix`] as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns this [`Prefix`] as a [`QName`] with the `xmlns` prefix. This is useful for serializing namespaces.
    pub fn xmlns(&self) -> QName<'_> {
        QName::new(Some(Prefix::XMLNS), <&LocalName>::from(self))
    }

    /// Returns `true` if this [`Prefix`] is the default prefix.
    pub fn is_default(&self) -> bool {
        self.0.is_empty()
    }

    /// The blank (default) prefix.
    pub const BLANK: &'static Prefix =
        //SAFETY: Hardcoded valid prefix.
        unsafe { Prefix::new_unchecked("") };

    /// The `xmlns` prefix used for XML namespace declarations.
    pub const XMLNS: &'static Prefix =
        //SAFETY: Hardcoded valid prefix.
        unsafe { Prefix::new_unchecked("xmlns") };
}

/// An owned version of [`Prefix`].
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrefixBuf(String);

impl Default for PrefixBuf {
    fn default() -> Self {
        Prefix::BLANK.to_owned()
    }
}

impl PrefixBuf {
    /// Creates a new [`PrefixBuf`] from a string without validating it.
    ///
    /// # Safety
    /// The caller must ensure that the value is a valid XML name.
    pub const unsafe fn new_unchecked(value: String) -> Self {
        Self(value)
    }

    /// Creates a new [`PrefixBuf`] from a string.
    pub fn new(value: String) -> Result<Self, PrefixParseError> {
        // Validate the value
        Prefix::new(&value)?;

        // SAFETY: The value has been validated.
        Ok(unsafe { Self::new_unchecked(value) })
    }
}

impl Borrow<Prefix> for PrefixBuf {
    fn borrow(&self) -> &Prefix {
        // SAFETY: All PrefixBufs are valid Prefixes.
        unsafe { Prefix::new_unchecked(&self.0) }
    }
}

impl Deref for PrefixBuf {
    type Target = Prefix;
    fn deref(&self) -> &Self::Target {
        self.borrow()
    }
}

impl ToOwned for Prefix {
    type Owned = PrefixBuf;
    fn to_owned(&self) -> Self::Owned {
        PrefixBuf(self.0.to_owned())
    }
}

impl FromStr for PrefixBuf {
    type Err = PrefixParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Prefix::new(s).map(ToOwned::to_owned)
    }
}

impl<'a> From<&'a Prefix> for &'a LocalName {
    fn from(value: &'a Prefix) -> Self {
        // SAFETY: All Prefixes are valid LocalNames.
        unsafe { LocalName::new_unchecked(value.as_str()) }
    }
}

impl<'a> From<Option<&'a Prefix>> for &'a Prefix {
    fn from(value: Option<&'a Prefix>) -> Self {
        value.unwrap_or(Prefix::BLANK)
    }
}

impl Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for Prefix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_text(&self.0)
    }
}

impl<'de> Deserialize<'de> for PrefixBuf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(types::string::FromStrVisitor::default())
    }
}

impl Display for PrefixBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl PartialEq<Prefix> for PrefixBuf {
    fn eq(&self, other: &Prefix) -> bool {
        self.deref() == other
    }
}

impl PartialEq<PrefixBuf> for Prefix {
    fn eq(&self, other: &PrefixBuf) -> bool {
        self == other.deref()
    }
}

/// # XML Local Name
/// A local name of an XML element or attribute within a [`XmlNamespace`].
///
/// Together with a [`XmlNamespace`], it forms an [`ExpandedName`].
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocalName(str);

/// An error that can occur when parsing a [`LocalName`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LocalNameParseError {
    /// The [`LocalName`] is not a valid XML name.
    #[error("Local name has an invalid XML name: {0}")]
    InvalidXmlName(#[from] name_tokens::InvalidXmlNameError),
}

impl LocalName {
    /// Creates a new [`LocalName`] from a string without validating it.
    ///
    /// # Safety
    /// The caller must ensure that the value is a valid XML name.
    pub const unsafe fn new_unchecked(value: &str) -> &Self {
        // SAFETY: The caller must ensure that the value is a valid XML name.
        unsafe { &*(value as *const str as *const LocalName) }
    }

    /// Creates a new [`LocalName`] from a string.
    pub fn new(value: &str) -> Result<&Self, LocalNameParseError> {
        name_tokens::is_valid_name(value)?;

        // SAFETY: The value has been validated.
        Ok(unsafe { Self::new_unchecked(value) })
    }

    /// Returns this [`LocalName`] as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The local name for XML namespace declarations with no prefix.
    pub const XMLNS: &'static LocalName =
        //SAFETY: Hardcoded valid local name.
        unsafe { LocalName::new_unchecked("xmlns") };
}

impl Display for LocalName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for LocalName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_text(&self.0)
    }
}

/// An owned version of [`LocalName`].
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocalNameBuf(String);

impl LocalNameBuf {
    /// Creates a new [`LocalNameBuf`] from a string without validating it.
    ///
    /// # Safety
    /// The caller must ensure that the value is a valid XML name.
    pub const unsafe fn new_unchecked(value: String) -> Self {
        Self(value)
    }

    /// Creates a new [`LocalNameBuf`] from a string.
    pub fn new(value: String) -> Result<Self, LocalNameParseError> {
        // Validate the value
        LocalName::new(&value)?;

        // SAFETY: The value has been validated.
        Ok(unsafe { Self::new_unchecked(value) })
    }
}

impl Borrow<LocalName> for LocalNameBuf {
    fn borrow(&self) -> &LocalName {
        // SAFETY: All LocalNameBufs are valid LocalNames.
        unsafe { LocalName::new_unchecked(&self.0) }
    }
}

impl Deref for LocalNameBuf {
    type Target = LocalName;
    fn deref(&self) -> &Self::Target {
        self.borrow()
    }
}

impl ToOwned for LocalName {
    type Owned = LocalNameBuf;
    fn to_owned(&self) -> Self::Owned {
        LocalNameBuf(self.0.to_owned())
    }
}

impl FromStr for LocalNameBuf {
    type Err = LocalNameParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        LocalName::new(s).map(ToOwned::to_owned)
    }
}

impl Serialize for LocalNameBuf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.deref().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LocalNameBuf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(types::string::FromStrVisitor::default())
    }
}

impl Display for LocalNameBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl PartialEq<LocalName> for LocalNameBuf {
    fn eq(&self, other: &LocalName) -> bool {
        self.deref() == other
    }
}

impl PartialEq<LocalNameBuf> for LocalName {
    fn eq(&self, other: &LocalNameBuf) -> bool {
        self == other.deref()
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
        let prefix = Prefix::new(prefix_text).unwrap();
        assert_eq!(prefix.to_string(), prefix_text);
        assert_eq!(prefix.as_str().to_string(), prefix_text);
    }

    #[rstest]
    #[case::empty("", PrefixParseError::InvalidXmlName(InvalidXmlNameError::Empty))]
    #[case::space("invalid prefix", PrefixParseError::InvalidXmlName(InvalidXmlNameError::InvalidChar { index: 7, character: ' ' }))]
    fn invalid_prefix_invalid_characters(
        #[case] prefix: &str,
        #[case] expected_error: PrefixParseError,
    ) {
        let error = Prefix::new(prefix).unwrap_err();
        assert_eq!(error, expected_error);
    }

    #[rstest]
    #[case::basic("localName")]
    fn test_local_name(#[case] local_name_text: &str) {
        let local_name = LocalNameBuf::from_str(local_name_text).unwrap();
        assert_eq!(local_name.to_string(), local_name_text);
    }

    #[rstest]
    #[case::empty("", LocalNameParseError::InvalidXmlName(InvalidXmlNameError::Empty))]
    #[case::space("invalid localName", LocalNameParseError::InvalidXmlName(InvalidXmlNameError::InvalidChar { index: 7, character: ' ' }))]
    fn invalid_local_name_invalid_characters(
        #[case] local_name: &str,
        #[case] expected_error: LocalNameParseError,
    ) {
        let error = LocalNameBuf::from_str(local_name).unwrap_err();
        assert_eq!(error, expected_error);
    }

    #[rstest]
    #[case::basic("localName", None, "localName")]
    #[case::with_namespace("localName", Some(XmlNamespace::new("http://example.com").unwrap()), "{http://example.com}localName")]
    fn test_expanded_name(
        #[case] local_name_text: &str,
        #[case] namespace: Option<&XmlNamespace>,
        #[case] expanded_name_text: &str,
    ) {
        let local_name = LocalName::new(local_name_text).unwrap();
        let expanded_name = ExpandedName::new(local_name, namespace.clone());
        assert_eq!(expanded_name.local_name(), local_name);
        assert_eq!(expanded_name.namespace(), &namespace);
        assert_eq!(expanded_name.to_string(), expanded_name_text);
        assert_eq!(expanded_name.local_name.as_str(), local_name_text);
    }

    #[rstest]
    #[case::basic("prefix:localName")]
    fn test_qname(#[case] qname_text: &str) {
        let qname = QNameBuf::from_str(qname_text).unwrap();
        assert_eq!(qname.to_string(), qname_text);
    }

    #[rstest]
    #[case::invalid_local_name("prefix:invalid localName", QNameParseError::InvalidLocalName(LocalNameParseError::InvalidXmlName(InvalidXmlNameError::InvalidChar { index: 7, character: ' ' })))]
    fn invalid_qname_invalid_characters(
        #[case] qname: &str,
        #[case] expected_error: QNameParseError,
    ) {
        let error = QNameBuf::from_str(qname).unwrap_err();
        assert_eq!(error, expected_error);
    }
}
