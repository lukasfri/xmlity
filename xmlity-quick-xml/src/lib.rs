//! # XMLity Quick XML
//!
//! This crate contains a reference implementation of the `xmlity` crate using the `quick-xml` crate. It is the intention to keep this crate up to date with the latest version of `quick-xml` and `xmlity`.

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct _ReadMeDocTests;

use ::quick_xml::name::ResolveResult;
use core::str;

use xmlity::{ser::IncludePrefix, ExpandedName, LocalName, Prefix, QName, XmlNamespace};

pub mod de;
pub mod ser;

pub use de::{from_str, Deserializer};
use quick_xml::name::{LocalName as QuickLocalName, Prefix as QuickPrefix, QName as QuickName};
pub use ser::{to_string, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Quick XML error: {0}")]
    QuickXml(#[from] quick_xml::Error),
    #[error("Attribute error: {0}")]
    AttrError(#[from] quick_xml::events::attributes::AttrError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unexpected: {0}")]
    Unexpected(xmlity::de::Unexpected),
    #[error("Custom: {0}")]
    Custom(String),
    #[error("Wrong name: expected {expected:?}, got {actual:?}")]
    WrongName {
        actual: Box<ExpandedName<'static>>,
        expected: Box<ExpandedName<'static>>,
    },
    #[error("Unknown child")]
    UnknownChild,
    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("Invalid string")]
    InvalidString,
    #[error("Missing field: {field}")]
    MissingField { field: String },
    #[error("No possible variant: {ident}")]
    NoPossibleVariant { ident: String },
    #[error("Missing data")]
    MissingData,
}

impl xmlity::de::Error for Error {
    fn custom<T: ToString>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }

    fn wrong_name(actual: &ExpandedName<'_>, expected: &ExpandedName<'_>) -> Self {
        Error::WrongName {
            actual: Box::new(actual.clone().into_owned()),
            expected: Box::new(expected.clone().into_owned()),
        }
    }

    fn unexpected_visit<T>(unexpected: xmlity::de::Unexpected, _expected: &T) -> Self {
        Error::Unexpected(unexpected)
    }

    fn missing_field(field: &str) -> Self {
        Error::MissingField {
            field: field.to_string(),
        }
    }

    fn no_possible_variant(ident: &str) -> Self {
        Error::NoPossibleVariant {
            ident: ident.to_string(),
        }
    }

    fn missing_data() -> Self {
        Error::MissingData
    }

    fn unknown_child() -> Self {
        Error::UnknownChild
    }

    fn invalid_string() -> Self {
        Error::InvalidString
    }
}

impl xmlity::ser::Error for Error {
    fn custom<T: ToString>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

pub trait HasQuickXmlAlternative {
    type QuickXmlAlternative;

    fn from_quick_xml(quick_xml: Self::QuickXmlAlternative) -> Self;
}

impl<'a> HasQuickXmlAlternative for QName<'a> {
    type QuickXmlAlternative = QuickName<'a>;

    fn from_quick_xml(quick_xml: Self::QuickXmlAlternative) -> Self {
        QName::new(
            quick_xml.prefix().map(Prefix::from_quick_xml),
            LocalName::from_quick_xml(quick_xml.local_name()),
        )
    }
}

impl<'a> HasQuickXmlAlternative for Prefix<'a> {
    type QuickXmlAlternative = QuickPrefix<'a>;
    fn from_quick_xml(quick_xml: Self::QuickXmlAlternative) -> Self {
        Self::new(str::from_utf8(quick_xml.into_inner()).expect("prefix should be valid utf8"))
            .expect("A quick xml prefix should be valid")
    }
}

impl<'a> HasQuickXmlAlternative for LocalName<'a> {
    type QuickXmlAlternative = QuickLocalName<'a>;
    fn from_quick_xml(quick_xml: Self::QuickXmlAlternative) -> Self {
        Self::new(str::from_utf8(quick_xml.into_inner()).expect("local name should be valid utf8"))
            .expect("A quick xml local name should be valid")
    }
}

pub struct OwnedQuickName(Vec<u8>);

impl OwnedQuickName {
    pub fn new(name: &QName<'_>) -> Self {
        Self(name.to_string().into_bytes())
    }

    pub fn as_ref(&self) -> QuickName<'_> {
        QuickName(&self.0[..])
    }
}

pub fn xml_namespace_from_resolve_result(value: ResolveResult<'_>) -> Option<XmlNamespace<'_>> {
    match value {
        ResolveResult::Bound(namespace) => Some(
            XmlNamespace::new(str::from_utf8(namespace.0).expect("namespace should be valid utf8"))
                .unwrap(),
        ),
        _ => None,
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Attribute<'a> {
    pub name: ExpandedName<'a>,
    pub value: String,
    pub enforce_prefix: IncludePrefix,
    pub preferred_prefix: Option<Prefix<'a>>,
}

impl<'a> Attribute<'a> {
    pub fn resolve(self, resolved_prefix: Option<Prefix<'a>>) -> ResolvedAttribute<'a> {
        ResolvedAttribute {
            name: self.name.to_q_name(resolved_prefix),
            value: self.value,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResolvedAttribute<'a> {
    pub name: QName<'a>,
    pub value: String,
}

fn declaration_into_attribute(xmlns: XmlnsDeclaration<'_>) -> ResolvedAttribute<'_> {
    ResolvedAttribute {
        name: XmlnsDeclaration::xmlns_qname(xmlns.prefix),
        value: xmlns.namespace.as_str().to_owned(),
    }
}

/// An XML namespace declaration/singular mapping from a prefix to a namespace.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct XmlnsDeclaration<'a> {
    pub prefix: Prefix<'a>,
    pub namespace: XmlNamespace<'a>,
}

impl<'a> XmlnsDeclaration<'a> {
    pub fn new(prefix: Prefix<'a>, namespace: XmlNamespace<'a>) -> Self {
        Self { prefix, namespace }
    }

    pub fn into_owned(self) -> XmlnsDeclaration<'static> {
        XmlnsDeclaration {
            prefix: self.prefix.into_owned(),
            namespace: self.namespace.into_owned(),
        }
    }

    pub fn prefix(&self) -> &Prefix<'a> {
        &self.prefix
    }

    pub fn namespace(&self) -> &XmlNamespace<'a> {
        &self.namespace
    }

    /// Returns the QName for the XML namespace declaration.
    pub fn xmlns_qname(prefix: Prefix<'_>) -> QName<'_> {
        if prefix.is_default() {
            QName::new(
                None,
                LocalName::new("xmlns").expect("xmlns is a valid local name"),
            )
        } else {
            QName::new(
                Some(Prefix::new("xmlns").expect("xmlns is a valid prefix")),
                LocalName::from(prefix),
            )
        }
    }
}
