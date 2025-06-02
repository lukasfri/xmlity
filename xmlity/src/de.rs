//! This module contains the [`Deserialize`], [`Deserializer`] and [`DeserializationGroup`] traits and associated types.

use std::{
    borrow::Cow,
    error::Error as StdError,
    fmt::{self, Debug, Display},
};

use crate::{ExpandedName, Prefix, XmlNamespace};

/// A trait for errors that can be returned by a [`Deserializer`].
pub trait Error: Sized + StdError {
    /// Error for when a custom error occurs during deserialization.
    fn custom<T>(msg: T) -> Self
    where
        T: Display;

    /// Error for when a name is expected to be a certain value, but it is not.
    fn wrong_name(name: &ExpandedName<'_>, expected: &ExpandedName<'_>) -> Self;

    /// Error for when a type is expected to be a certain type, but it is not.
    fn unexpected_visit<T>(unexpected: Unexpected, expected: &T) -> Self;

    /// Error for when a field is missing.
    fn missing_field(field: &str) -> Self;

    /// Error for when a type has no possible variants to deserialize into.
    fn no_possible_variant(ident: &str) -> Self;

    /// Error for when a type is missing data that is required to deserialize it.
    fn missing_data() -> Self;

    /// Error for when a child cannot be identified, and ignoring it is not allowed.
    fn unknown_child() -> Self;

    /// Error for when a string is invalid for the type.
    fn invalid_string() -> Self;
}

/// An enum representing the unexpected type of data that was encountered.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Unexpected {
    /// A text node.
    #[error("text")]
    Text,
    /// A CDATA section.
    #[error("cdata")]
    CData,
    /// A sequence of XML values.
    #[error("sequence")]
    Seq,
    /// An element start.
    #[error("element start")]
    ElementStart,
    /// An element end.
    #[error("element end")]
    ElementEnd,
    /// An attribute.
    #[error("attribute")]
    Attribute,
    /// A comment.
    #[error("comment")]
    Comment,
    /// A declaration.
    #[error("declaration")]
    Decl,
    /// A processing instruction.
    #[error("processing instruction")]
    PI,
    /// A doctype.
    #[error("doctype")]
    DocType,
    /// End of file.
    #[error("eof")]
    Eof,
    /// Nothing.
    #[error("none")]
    None,
}

/// Trait that lets you access the namespaces declared on an XML node.
pub trait NamespaceContext {
    /// Get the default namespace.
    fn default_namespace(&self) -> Option<XmlNamespace<'_>>;

    /// Resolve a prefix to a namespace.
    fn resolve_prefix(&self, prefix: Prefix<'_>) -> Option<XmlNamespace<'_>>;
}

/// Trait that lets you access the attributes of an XML node.
pub trait AttributesAccess<'de> {
    /// The error type for this attributes access.
    type Error: Error;
    /// The type of the sub access for this attributes access returned by [`AttributesAccess::sub_access`].
    type SubAccess<'a>: AttributesAccess<'de, Error = Self::Error> + 'a
    where
        Self: 'a;

    /// Get the next attribute.
    fn next_attribute<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>;

    /// Get a sub access to the attributes.
    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error>;
}

impl<'de, T: AttributesAccess<'de>> AttributesAccess<'de> for &mut T {
    type Error = T::Error;
    type SubAccess<'a>
        = T::SubAccess<'a>
    where
        Self: 'a;

    fn next_attribute<D>(&mut self) -> Result<Option<D>, Self::Error>
    where
        D: Deserialize<'de>,
    {
        (*self).next_attribute()
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        (*self).sub_access()
    }
}

/// A trait for accessing properties of an element. This is the first stage of element deserialization, where the element's name and attributes are accessed. The second stage is accessing the element's children, which is done by calling [`ElementAccess::children`].
pub trait ElementAccess<'de>: AttributesAccess<'de> {
    /// The type of the children accessor returned by [`ElementAccess::children`].
    type ChildrenAccess: SeqAccess<'de, Error = Self::Error>;

    /// The type of the namespace context returned by [`AttributeAccess::namespace_context`].
    type NamespaceContext<'a>: NamespaceContext + 'a
    where
        Self: 'a;

    /// Returns the name of the element.
    fn name(&self) -> ExpandedName<'_>;

    /// Returns an accessor for the element's children.
    fn children(self) -> Result<Self::ChildrenAccess, Self::Error>;

    /// Returns the namespace context for this attribute.
    fn namespace_context(&self) -> Self::NamespaceContext<'_>;
}

/// An extension trait for [`ElementAccess`] that provides additional methods.
pub trait ElementAccessExt<'de>: ElementAccess<'de> {
    /// Ensures that the element has the given name. If it does not, returns an error.
    fn ensure_name<E: Error>(&self, name: &ExpandedName) -> Result<(), E>;
}

impl<'de, T: ElementAccess<'de>> ElementAccessExt<'de> for T {
    fn ensure_name<E: Error>(&self, name: &ExpandedName) -> Result<(), E> {
        if self.name() == *name {
            Ok(())
        } else {
            Err(Error::wrong_name(&self.name(), name))
        }
    }
}

/// A trait for accessing properties of an attribute.
pub trait AttributeAccess<'de> {
    /// The error type for this attribute access.
    type Error: Error;

    /// Returns the name of the attribute.
    fn name(&self) -> ExpandedName<'_>;

    /// Deserializes the value of the attribute.
    fn value<T>(self) -> Result<T, Self::Error>
    where
        T: Deserialize<'de>;
}

/// An extension trait for [`AttributeAccess`] that provides additional methods.
pub trait AttributeAccessExt<'de>: AttributeAccess<'de> {
    /// Ensures that the attribute has the given name.
    fn ensure_name<E: Error>(&self, name: &ExpandedName) -> Result<(), E>;
}

impl<'de, T: AttributeAccess<'de>> AttributeAccessExt<'de> for T {
    fn ensure_name<E: Error>(&self, name: &ExpandedName) -> Result<(), E> {
        if self.name() == *name {
            Ok(())
        } else {
            Err(Error::wrong_name(&self.name(), name))
        }
    }
}

/// A trait for accessing a sequence of nodes, which could include a mix of elements and text nodes.
pub trait SeqAccess<'de> {
    /// The error type for this sequence access.
    type Error: Error;

    /// The type of the sub-access for this sequence access returned by [`SeqAccess::sub_access`].
    type SubAccess<'g>: SeqAccess<'de, Error = Self::Error>
    where
        Self: 'g;

    /// Gets the next element in the sequence.
    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>;

    /// Gets the next element by trying to deserialize it as a sequence.
    fn next_element_seq<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize<'de>;

    /// Gets the sub-access for the current sequence access.
    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error>;
}

impl<'de, T: SeqAccess<'de>> SeqAccess<'de> for &mut T {
    type Error = T::Error;
    type SubAccess<'g>
        = T::SubAccess<'g>
    where
        Self: 'g;

    fn next_element<U>(&mut self) -> Result<Option<U>, Self::Error>
    where
        U: Deserialize<'de>,
    {
        (*self).next_element()
    }

    fn next_element_seq<U>(&mut self) -> Result<Option<U>, Self::Error>
    where
        U: Deserialize<'de>,
    {
        (*self).next_element_seq()
    }

    fn sub_access(&mut self) -> Result<Self::SubAccess<'_>, Self::Error> {
        (*self).sub_access()
    }
}

/// Trait for XML text.
pub trait XmlText<'de> {
    /// The type of the namespace context returned by [`AttributeAccess::namespace_context`].
    type NamespaceContext<'a>: NamespaceContext + 'a
    where
        Self: 'a;

    /// Returns the owned byte representation of the text.
    fn into_bytes(self) -> Cow<'de, [u8]>;

    /// Returns the byte representation of the text.
    fn as_bytes(&self) -> &[u8];

    /// Returns the owned string representation of the text.
    fn into_string(self) -> Cow<'de, str>;

    /// Returns the string representation of the text.
    fn as_str(&self) -> &str;

    /// Returns the namespace context of the text.
    fn namespace_context(&self) -> Self::NamespaceContext<'_>;
}

/// Trait for XML CDATA.
pub trait XmlCData<'de> {
    /// The type of the namespace context returned by [`AttributeAccess::namespace_context`].
    type NamespaceContext<'a>: NamespaceContext + 'a
    where
        Self: 'a;

    /// Returns the owned byte representation of the CDATA.
    fn into_bytes(self) -> Cow<'de, [u8]>;

    /// Returns the byte representation of the CDATA.
    fn as_bytes(&self) -> &[u8];

    /// Returns the owned string representation of the CDATA.
    fn into_string(self) -> Cow<'de, str>;

    /// Returns the string representation of the CDATA.
    fn as_str(&self) -> &str;

    /// Returns the namespace context of the CDATA.
    fn namespace_context(&self) -> Self::NamespaceContext<'_>;
}

/// Trait for XML processing instructions.
pub trait XmlProcessingInstruction {
    /// The type of the namespace context returned by [`AttributeAccess::namespace_context`].
    type NamespaceContext<'a>: NamespaceContext + 'a
    where
        Self: 'a;

    /// Returns the target of the PI.
    fn target(&self) -> &[u8];

    /// Returns the content of the PI.
    fn content(&self) -> &[u8];

    /// Returns the namespace context of the PI.
    fn namespace_context(&self) -> Self::NamespaceContext<'_>;
}

/// Trait for XML declarations.
pub trait XmlDeclaration {
    /// The type of the namespace context returned by [`AttributeAccess::namespace_context`].
    type NamespaceContext<'a>: NamespaceContext + 'a
    where
        Self: 'a;

    /// Returns the version value of the declaration.
    fn version(&self) -> &[u8];

    /// Returns the encoding value of the declaration.
    fn encoding(&self) -> Option<&[u8]>;

    /// Returns the standalone value of the declaration.
    fn standalone(&self) -> Option<&[u8]>;

    /// Returns the namespace context of the declaration.
    fn namespace_context(&self) -> Self::NamespaceContext<'_>;
}

/// Trait for XML comments.
pub trait XmlComment<'de> {
    /// The type of the namespace context returned by [`AttributeAccess::namespace_context`].
    type NamespaceContext<'a>: NamespaceContext + 'a
    where
        Self: 'a;

    /// Returns the owned byte representation of the comment.
    fn into_bytes(self) -> Cow<'de, [u8]>;

    /// Returns the byte representation of the comment.
    fn as_bytes(&self) -> &[u8];

    /// Returns the namespace context of the comment.
    fn namespace_context(&self) -> Self::NamespaceContext<'_>;
}

/// Trait for XML doctypes.
pub trait XmlDoctype<'de> {
    /// The type of the namespace context returned by [`AttributeAccess::namespace_context`].
    type NamespaceContext<'a>: NamespaceContext + 'a
    where
        Self: 'a;

    /// Returns the owned byte representation of the doctype.
    fn into_bytes(self) -> Cow<'de, [u8]>;

    /// Returns the byte representation of the doctype.
    fn as_bytes(&self) -> &[u8];

    /// Returns the namespace context of the doctype.
    fn namespace_context(&self) -> Self::NamespaceContext<'_>;
}

/// Visitor trait that lets you define how to handle different types of XML nodes.
pub trait Visitor<'de>: Sized {
    /// The type of value that this visitor will produce.
    type Value: Deserialize<'de>;

    /// Returns a description of the type that this visitor expects.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result;

    /// Visits an XML text node.
    fn visit_text<E, V>(self, value: V) -> Result<Self::Value, E>
    where
        E: Error,
        V: XmlText<'de>,
    {
        let _ = value;
        Err(Error::unexpected_visit(Unexpected::Text, &self))
    }

    /// Visits an XML CDATA node.
    fn visit_cdata<E, V>(self, value: V) -> Result<Self::Value, E>
    where
        E: Error,
        V: XmlCData<'de>,
    {
        let _ = value;
        Err(Error::unexpected_visit(Unexpected::CData, &self))
    }

    /// Visits an XML element.
    fn visit_element<A>(self, element: A) -> Result<Self::Value, A::Error>
    where
        A: ElementAccess<'de>,
    {
        let _ = element;
        Err(Error::unexpected_visit(Unexpected::ElementStart, &self))
    }

    /// Visits an XML attribute.
    fn visit_attribute<A>(self, attribute: A) -> Result<Self::Value, A::Error>
    where
        A: AttributeAccess<'de>,
    {
        let _ = attribute;
        Err(Error::unexpected_visit(Unexpected::Attribute, &self))
    }

    /// Visits a sequence of values.
    fn visit_seq<S>(self, mut sequence: S) -> Result<Self::Value, S::Error>
    where
        S: SeqAccess<'de>,
    {
        sequence
            .next_element::<Self::Value>()?
            .ok_or_else(Error::missing_data)
    }

    /// Visits an XML PI node.
    fn visit_pi<E, V>(self, pi: V) -> Result<Self::Value, E>
    where
        E: Error,
        V: XmlProcessingInstruction,
    {
        let _ = pi;
        Err(Error::unexpected_visit(Unexpected::PI, &self))
    }

    /// Visits a declaration.
    fn visit_decl<E, V>(self, declaration: V) -> Result<Self::Value, E>
    where
        E: Error,
        V: XmlDeclaration,
    {
        let _ = declaration;
        Err(Error::unexpected_visit(Unexpected::Decl, &self))
    }

    /// Visits a comment.
    fn visit_comment<E, V>(self, comment: V) -> Result<Self::Value, E>
    where
        E: Error,
        V: XmlComment<'de>,
    {
        let _ = comment;
        Err(Error::unexpected_visit(Unexpected::Comment, &self))
    }

    /// Visits a doctype declaration.
    fn visit_doctype<E, V>(self, doctype: V) -> Result<Self::Value, E>
    where
        E: Error,
        V: XmlDoctype<'de>,
    {
        let _ = doctype;
        Err(Error::unexpected_visit(Unexpected::DocType, &self))
    }

    /// Visits nothing. This is used when a value is not present.
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Err(Error::unexpected_visit(Unexpected::None, &self))
    }
}

/// A type that can be used to deserialize XML documents.
pub trait Deserializer<'de>: Sized {
    /// The error type that can be returned from the deserializer.
    type Error: Error;

    /// Deserializes a value from the deserializer.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>;

    /// Deserializes a value from the deserializer, but tries to do it from a sequence of values.
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>;
}

/// A type that can be deserialized from a deserializer. This type has two methods: [`Deserialize::deserialize`] and [`Deserialize::deserialize_seq`]. The latter is used in cases where types can be constructed from multiple nodes, such as constructing a [`std::vec::Vec`] from multiple elements, or a [`std::string::String`] from multiple text nodes that are concatenated together.
///
/// To see the documentation for the derive macro, see [`xmlity_derive::Deserialize`].
pub trait Deserialize<'de>: Sized {
    /// Deserializes a value from a deserializer.
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error>;

    /// Deserializes a value from a deserializer, but tries to do it from a sequence of values.
    fn deserialize_seq<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        Self::deserialize(reader)
    }
}

/// A utility type for easier use of [`Deserialize`] trait without needing to specify the lifetime.
pub trait DeserializeOwned: for<'de> Deserialize<'de> {}
impl<T> DeserializeOwned for T where T: for<'de> Deserialize<'de> {}

/// A group of types that can be deserialized together. While this is being built, the type of a [`DeserializationGroup`] is the [`DeserializationGroup::Builder`] type.
///
/// To see the documentation for the derive macro, see [`xmlity_derive::DeserializationGroup`].
pub trait DeserializationGroup<'de> {
    /// The type of the builder for this deserialization group returned by [`DeserializationGroup::builder`].
    type Builder: DeserializationGroupBuilder<'de, Value = Self>;

    /// Initializes the deserialization group builder.
    fn builder() -> Self::Builder;
}

/// A builder for a deserialization group. When completed (through [`DeserializationGroupBuilder::finish`]), the builder is converted into the deserialization group type that initated the builder.
pub trait DeserializationGroupBuilder<'de>: Sized {
    /// The type of the deserialization group that this builder builds when finished through [`DeserializationGroupBuilder::finish`].
    type Value;

    /// Returns true if the deserializer made progress
    fn contribute_attributes<D: AttributesAccess<'de>>(
        &mut self,
        access: D,
    ) -> Result<bool, D::Error> {
        let _ = access;

        Ok(false)
    }

    /// This hint function is used to avoid calling [`DeserializationGroupBuilder::contribute_attributes`] unnecessarily.
    fn attributes_done(&self) -> bool {
        false
    }

    /// Returns true if the deserializer made progress
    fn contribute_elements<D: SeqAccess<'de>>(&mut self, access: D) -> Result<bool, D::Error> {
        let _ = access;

        Ok(false)
    }

    /// This hint function is used to avoid calling [`DeserializationGroupBuilder::contribute_elements`] unnecessarily.
    fn elements_done(&self) -> bool {
        false
    }

    /// This function is called after all attributes and elements have been contributed.
    fn finish<E: Error>(self) -> Result<Self::Value, E>;
}
