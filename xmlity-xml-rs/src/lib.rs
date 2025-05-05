//! # XMLity Quick XML
//!
//! This crate contains a reference implementation of the `xmlity` crate using the `quick-xml` crate. It is the intention to keep this crate up to date with the latest version of `quick-xml` and `xmlity`.
#[cfg(doctest)]
#[doc = include_str!("../../README.md")]
struct _RootReadMeDocTests;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct _ReadMeDocTests;

use xmlity::{ser::IncludePrefix, ExpandedName, LocalName, Prefix, QName, XmlNamespace};

pub mod de;
pub mod ser;

pub use de::Deserializer;
pub use ser::{to_string, Serializer};
use xml::name::OwnedName;

pub trait HasXmlRsAlternative {
    type XmlityEquivalent;

    fn into_xmlity(self) -> Self::XmlityEquivalent;
}

impl HasXmlRsAlternative for OwnedName {
    type XmlityEquivalent = QName<'static>;

    fn into_xmlity(self) -> Self::XmlityEquivalent {
        QName::new(
            self.prefix
                .map(|prefix| Prefix::new(prefix).expect("A xml-rs prefix should be valid")),
            LocalName::new(self.local_name).expect("An xml-rs local name should be valid"),
        )
    }
}

impl<'a> HasXmlRsAlternative for &'a OwnedName {
    type XmlityEquivalent = QName<'a>;

    fn into_xmlity(self) -> Self::XmlityEquivalent {
        QName::new(
            self.prefix
                .as_deref()
                .map(|prefix| Prefix::new(prefix).expect("A xml-rs prefix should be valid")),
            LocalName::new(self.local_name.as_str()).expect("An xml-rs local name should be valid"),
        )
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
