#![warn(missing_docs)]
//! # XMLity Quick XML
//!
//! This crate contains a reference implementation of the `xmlity` crate using the `quick-xml` crate. It is the intention to keep this crate up to date with the latest version of `quick-xml` and `xmlity`.
#[cfg(doctest)]
#[doc = include_str!("../../README.md")]
struct _RootReadMeDocTests;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct _ReadMeDocTests;

use ::quick_xml::name::ResolveResult;
use core::str;

use xmlity::{LocalName, Prefix, QName, XmlNamespace};

/// Includes the deserializer for the `quick-xml` crate.
pub mod de;
/// Includes the serializer for the `quick-xml` crate.
pub mod ser;

pub use de::{from_str, Deserializer};
use quick_xml::name::{LocalName as QuickLocalName, Prefix as QuickPrefix, QName as QuickName};
pub use ser::{to_string, to_string_pretty, Serializer};

trait HasQuickXmlAlternative {
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

struct OwnedQuickName(Vec<u8>);

impl OwnedQuickName {
    pub fn new(name: &QName<'_>) -> Self {
        Self(name.to_string().into_bytes())
    }

    pub fn as_ref(&self) -> QuickName<'_> {
        QuickName(&self.0[..])
    }
}

fn xml_namespace_from_resolve_result(value: ResolveResult<'_>) -> Option<XmlNamespace<'_>> {
    match value {
        ResolveResult::Bound(namespace) => Some(
            XmlNamespace::new(str::from_utf8(namespace.0).expect("namespace should be valid utf8"))
                .unwrap(),
        ),
        _ => None,
    }
}

/// An XML namespace declaration/singular mapping from a prefix to a namespace.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct XmlnsDeclaration<'a> {
    pub prefix: Prefix<'a>,
    pub namespace: XmlNamespace<'a>,
}

impl<'a> XmlnsDeclaration<'a> {
    pub fn new(prefix: Prefix<'a>, namespace: XmlNamespace<'a>) -> Self {
        Self { prefix, namespace }
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
