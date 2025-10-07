//! This module contains the deserialization implementations for the XML value types including visitors.
use crate::{
    de::{self, DeserializeContext, SeqAccess, Visitor},
    Deserialize, Deserializer, ExpandedName, Prefix, XmlNamespace,
};
use core::marker::PhantomData;

use super::*;

impl<'de> Deserialize<'de> for XmlValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        XmlValueWithoutSeq::deserialize(deserializer).map(From::from)
    }

    fn deserialize_seq<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_seq(IteratorVisitor::<_, XmlSeq<XmlValueWithoutSeq>>::default())
            .map(|mut a| match a.values.len() {
                0 => XmlValue::None,
                1 => a.values.pop_front().expect("Just checked.").into(),
                _ => XmlValue::Seq(a.values.into_iter().map(From::from).collect()),
            })
    }
}

impl<'de> Deserialize<'de> for XmlValueWithoutSeq {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct __Visitor<'v> {
            marker: PhantomData<XmlValueWithoutSeq>,
            lifetime: PhantomData<&'v ()>,
        }

        impl<'v> crate::de::Visitor<'v> for __Visitor<'v> {
            type Value = XmlValueWithoutSeq;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a comment")
            }

            fn visit_text<E, V>(self, value: V) -> Result<Self::Value, E>
            where
                E: de::Error,
                V: de::XmlText<'v>,
            {
                XmlTextVisitor::new()
                    .visit_text(value)
                    .map(XmlValueWithoutSeq::Text)
            }

            fn visit_cdata<E, V>(self, value: V) -> Result<Self::Value, E>
            where
                E: de::Error,
                V: de::XmlCData<'v>,
            {
                XmlCDataVisitor::new()
                    .visit_cdata(value)
                    .map(XmlValueWithoutSeq::CData)
            }

            fn visit_element<A>(self, element: A) -> Result<Self::Value, A::Error>
            where
                A: de::ElementAccess<'v>,
            {
                XmlElementVisitor::new()
                    .visit_element(element)
                    .map(XmlValueWithoutSeq::Element)
            }

            fn visit_pi<E, V>(self, value: V) -> Result<Self::Value, E>
            where
                E: de::Error,
                V: de::XmlProcessingInstruction,
            {
                XmlProcessingInstructionVisitor::new()
                    .visit_pi(value)
                    .map(XmlValueWithoutSeq::PI)
            }

            fn visit_decl<E, V>(self, declaration: V) -> Result<Self::Value, E>
            where
                E: de::Error,
                V: de::XmlDeclaration,
            {
                XmlDeclVisitor::new()
                    .visit_decl(declaration)
                    .map(XmlValueWithoutSeq::Decl)
            }

            fn visit_comment<E, V>(self, comment: V) -> Result<Self::Value, E>
            where
                E: de::Error,
                V: de::XmlComment<'v>,
            {
                XmlCommentVisitor::new()
                    .visit_comment(comment)
                    .map(XmlValueWithoutSeq::Comment)
            }

            fn visit_doctype<E, V>(self, value: V) -> Result<Self::Value, E>
            where
                E: de::Error,
                V: de::XmlDoctype<'v>,
            {
                XmlDoctypeVisitor::new()
                    .visit_doctype(value)
                    .map(XmlValueWithoutSeq::Doctype)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(XmlValueWithoutSeq::None)
            }
        }

        deserializer.deserialize_any(__Visitor {
            lifetime: PhantomData,
            marker: PhantomData,
        })
    }
}

// Text

/// A visitor for deserializing to [`XmlText`].
pub struct XmlTextVisitor<'v> {
    marker: PhantomData<XmlText>,
    lifetime: PhantomData<&'v ()>,
}

impl XmlTextVisitor<'_> {
    /// Creates a new [`XmlTextVisitor`].
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
            lifetime: PhantomData,
        }
    }
}

impl Default for XmlTextVisitor<'_> {
    fn default() -> Self {
        Self::new()
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

impl DeserializeContext for () {
    fn default_namespace(&self) -> Option<&XmlNamespace> {
        None
    }

    fn resolve_prefix(&self, _prefix: &Prefix) -> Option<&XmlNamespace> {
        None
    }

    fn external_data<T>(&self) -> Option<&T>
    where
        T: core::any::Any,
    {
        None
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

// CData

/// A visitor for deserializing to [`XmlCData`].
pub struct XmlCDataVisitor<'v> {
    marker: PhantomData<XmlCData>,
    lifetime: PhantomData<&'v ()>,
}
impl XmlCDataVisitor<'_> {
    /// Creates a new [`XmlCDataVisitor`].
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
            lifetime: PhantomData,
        }
    }
}

impl Default for XmlCDataVisitor<'_> {
    fn default() -> Self {
        Self::new()
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
        V: de::XmlCData<'de>,
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

// Child

/// A visitor for deserializing to [`XmlChild`].
pub struct XmlChildVisitor<'v> {
    marker: PhantomData<XmlChild>,
    lifetime: PhantomData<&'v ()>,
}
impl XmlChildVisitor<'_> {
    /// Creates a new [`XmlChildVisitor`].
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
            lifetime: PhantomData,
        }
    }
}

impl Default for XmlChildVisitor<'_> {
    fn default() -> Self {
        Self::new()
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
        V: de::XmlCData<'v>,
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
        V: de::XmlComment<'v>,
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

// Element

/// A visitor for deserializing to [`XmlElement`].
pub struct XmlElementVisitor<'v> {
    marker: PhantomData<XmlElement>,
    lifetime: PhantomData<&'v ()>,
}

impl XmlElementVisitor<'_> {
    /// Creates a new [`XmlElementVisitor`].
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
            lifetime: PhantomData,
        }
    }
}

impl Default for XmlElementVisitor<'_> {
    fn default() -> Self {
        Self::new()
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
        let name = element.name().into_owned();
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

// Attribute

/// A visitor for deserializing to [`XmlAttribute`].
pub struct XmlAttributeVisitor<'v> {
    marker: PhantomData<XmlAttribute>,
    lifetime: PhantomData<&'v ()>,
}

impl XmlAttributeVisitor<'_> {
    /// Creates a new [`XmlAttributeVisitor`].
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
            lifetime: PhantomData,
        }
    }
}

impl Default for XmlAttributeVisitor<'_> {
    fn default() -> Self {
        Self::new()
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
            name: attribute.name().into_owned(),
            value: attribute.value()?,
        })
    }
}

impl<'a> de::AttributeAccess<'a> for &'a XmlAttribute {
    type Error = XmlValueDeserializerError;

    fn name(&self) -> ExpandedName<'_> {
        self.name.as_ref()
    }

    fn value<T>(self) -> Result<T, Self::Error>
    where
        T: Deserialize<'a>,
    {
        T::deserialize(&self.value)
    }
}

impl<'de> crate::de::Deserialize<'de> for XmlAttribute {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: crate::de::Deserializer<'de>,
    {
        deserializer.deserialize_any(XmlAttributeVisitor::new())
    }
}

// Seq

impl<'de, T: Deserialize<'de>> Deserialize<'de> for XmlSeq<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: crate::de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(IteratorVisitor::<_, Self>::default())
    }
}

// Processing Instruction

/// A visitor for deserializing to [`XmlProcessingInstruction`].
pub struct XmlProcessingInstructionVisitor<'v> {
    marker: PhantomData<XmlProcessingInstruction>,
    lifetime: PhantomData<&'v ()>,
}

impl XmlProcessingInstructionVisitor<'_> {
    /// Creates a new [`XmlProcessingInstructionVisitor`].
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
            lifetime: PhantomData,
        }
    }
}

impl Default for XmlProcessingInstructionVisitor<'_> {
    fn default() -> Self {
        Self::new()
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

// Xml Decl

/// A visitor for deserializing to [`XmlDeclVisitor`].
pub struct XmlDeclVisitor<'v> {
    marker: PhantomData<XmlDecl>,
    lifetime: PhantomData<&'v ()>,
}

impl XmlDeclVisitor<'_> {
    /// Creates a new [`XmlDeclVisitor`].
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
            lifetime: PhantomData,
        }
    }
}

impl Default for XmlDeclVisitor<'_> {
    fn default() -> Self {
        Self::new()
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

// Xml Comment

/// A visitor for deserializing to [`XmlComment`].
pub struct XmlCommentVisitor<'v> {
    marker: PhantomData<XmlComment>,
    lifetime: PhantomData<&'v ()>,
}

impl XmlCommentVisitor<'_> {
    /// Creates a new [`XmlCommentVisitor`].
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
            lifetime: PhantomData,
        }
    }
}

impl Default for XmlCommentVisitor<'_> {
    fn default() -> Self {
        Self::new()
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
        V: de::XmlComment<'v>,
    {
        Ok(XmlComment(comment.into_bytes().into_owned()))
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

// Xml Doctype

/// A visitor for deserializing to [`XmlDoctype`].
pub struct XmlDoctypeVisitor<'v> {
    marker: PhantomData<XmlDoctype>,
    lifetime: PhantomData<&'v ()>,
}

impl XmlDoctypeVisitor<'_> {
    /// Creates a new [`XmlDoctypeVisitor`].
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
            lifetime: PhantomData,
        }
    }
}

impl Default for XmlDoctypeVisitor<'_> {
    fn default() -> Self {
        Self::new()
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
        V: de::XmlDoctype<'v>,
    {
        Ok(XmlDoctype(value.into_bytes().into_owned()))
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
