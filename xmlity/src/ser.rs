//! This module contains the [`Serialize`], [`SerializeAttribute`], [`Serializer`] and [`SerializationGroup`] traits and associated types.
use std::fmt::Display;

use crate::{ExpandedName, Prefix};

/// A trait for errors that can be returned by serializer after a serialization attempt.
pub trait Error {
    /// Creates an error with a custom message.
    fn custom<T>(msg: T) -> Self
    where
        T: Display;
}

/// A trait for serializing attributes.
pub trait SerializeAttributes: Sized {
    /// The type of the value that is returned when serialization is successful.
    type Ok;
    /// The type of the error that is returned when serialization fails.
    type Error: Error;

    /// Serializes an attribute.
    fn serialize_attribute<A: SerializeAttribute>(
        &mut self,
        a: &A,
    ) -> Result<Self::Ok, Self::Error>;
}

impl<T: SerializeAttributes> SerializeAttributes for &mut T {
    type Ok = T::Ok;
    type Error = T::Error;

    fn serialize_attribute<A: SerializeAttribute>(
        &mut self,
        a: &A,
    ) -> Result<Self::Ok, Self::Error> {
        SerializeAttributes::serialize_attribute(*self, a)
    }
}

/// A trait for serializing elements.
#[must_use = "serializers are lazy and must be consumed to perform serialization. Try calling `.end()` on the serializer."]
pub trait SerializeElement: SerializeAttributes {
    /// The type of the value that is returned when serialization is successful.
    type SerializeElementChildren: SerializeElementChildren<Ok = Self::Ok, Error = Self::Error>;

    /// Always serialize this element with the given prefix.
    fn include_prefix(&mut self, should_enforce: IncludePrefix) -> Result<Self::Ok, Self::Error>;

    /// Set the preferred prefix for this element.
    fn preferred_prefix(
        &mut self,
        preferred_prefix: Option<Prefix<'_>>,
    ) -> Result<Self::Ok, Self::Error>;

    /// Serialize the children of this element.
    fn serialize_children(self) -> Result<Self::SerializeElementChildren, Self::Error>;

    /// End the serialization of this element with no children.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// A trait for serializing children of an element.
#[must_use = "serializers are lazy and must be consumed to perform serialization. Try calling `.end()` on the serializer."]
pub trait SerializeChildren: Sized {
    /// The type of the value that is returned when serialization is successful.
    type Ok;
    /// The type of the error that is returned when serialization fails.
    type Error: Error;

    /// Serialize a child of this element.
    fn serialize_child<V: Serialize>(&mut self, v: &V) -> Result<Self::Ok, Self::Error>;
}

impl<S: SerializeChildren> SerializeChildren for &mut S {
    type Ok = S::Ok;
    type Error = S::Error;
    fn serialize_child<V: Serialize>(&mut self, v: &V) -> Result<Self::Ok, Self::Error> {
        S::serialize_child(self, v)
    }
}

/// A trait for ending the serialization of an element with children.
#[must_use = "serializers are lazy and must be consumed to perform serialization. Try calling `.end()` on the serializer."]
pub trait SerializeElementChildren: SerializeChildren {
    /// End the serialization of this element with children.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// A trait for serializing a sequence of elements.
#[must_use = "serializers are lazy and must be consumed to perform serialization. Try calling `.end()` on the serializer."]
pub trait SerializeSeq {
    /// The type of the value that is returned when serialization is successful.
    type Ok;
    /// The type of the error that is returned when serialization fails.
    type Error: Error;

    /// Serialize an element in the sequence.
    fn serialize_element<V: Serialize>(&mut self, v: &V) -> Result<Self::Ok, Self::Error>;

    /// End the serialization of the sequence.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// A serializer receives serialization instructions from a [`Serialize`] implementation and produces serialized output.
pub trait Serializer: Sized {
    /// The type of the value that is returned when serialization is successful.
    type Ok;
    /// The type of the error that is returned when serialization fails.
    type Error: Error;
    /// The type of the serializer that is used to serialize an element with children.
    type SerializeElement: SerializeElement<Ok = Self::Ok, Error = Self::Error>;
    /// The type of the serializer that is used to serialize a sequence of elements.
    type SerializeSeq: SerializeSeq<Ok = Self::Ok, Error = Self::Error>;

    /// Serialize a text node.
    fn serialize_text<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error>;

    /// Serialize a CDATA section.
    fn serialize_cdata<S: AsRef<str>>(self, text: S) -> Result<Self::Ok, Self::Error>;

    /// Serialize an element with children.
    fn serialize_element(
        self,
        name: &'_ ExpandedName<'_>,
    ) -> Result<Self::SerializeElement, Self::Error>;

    /// Serialize a sequence of elements.
    fn serialize_seq(self) -> Result<Self::SerializeSeq, Self::Error>;

    /// Serialize an XML declaration.
    fn serialize_decl<S: AsRef<str>>(
        self,
        version: S,
        encoding: Option<S>,
        standalone: Option<S>,
    ) -> Result<Self::Ok, Self::Error>;

    /// Serialize a processing instruction.
    fn serialize_pi<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error>;

    /// Serialize a comment.
    fn serialize_comment<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error>;

    /// Serialize a doctype declaration.
    fn serialize_doctype<S: AsRef<[u8]>>(self, text: S) -> Result<Self::Ok, Self::Error>;

    /// Serialize nothing.
    fn serialize_none(self) -> Result<Self::Ok, Self::Error>;
}

/// A type that can serialize attributes. Works in a similar way to [`Serializer`].
pub trait AttributeSerializer: Sized {
    /// The type of the value that is returned when serialization is successful.
    type Ok;
    /// The type of the error that is returned when serialization fails.
    type Error: Error;
    /// The type returned when serializing an attribute.
    type SerializeAttribute<'a>: SerializeAttributeAccess<Ok = Self::Ok, Error = Self::Error>
    where
        Self: 'a;

    /// Serialize an attribute.
    fn serialize_attribute(
        &mut self,
        name: &'_ ExpandedName<'_>,
    ) -> Result<Self::SerializeAttribute<'_>, Self::Error>;

    /// Serialize nothing.
    fn serialize_none(&mut self) -> Result<Self::Ok, Self::Error>;
}

/// A type that can be serialized. To serialize, you provide it with a [`Serializer`] that then gets instructions from the type on how to serialize itself.
///
/// To see the documentation for the derive macro, see [`xmlity_derive::Serialize`].
pub trait Serialize {
    /// Serialize the type.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
}

impl<T: Serialize> Serialize for &T {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        T::serialize(*self, serializer)
    }
}

impl<T: Serialize> Serialize for &mut T {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        T::serialize(*self, serializer)
    }
}

/// Setting for whether to enforce a prefix when serializing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IncludePrefix {
    /// Always enforce the prefix.
    Always,
    /// Only when the preferred prefix is not the used prefix.
    WhenNecessaryForPreferredPrefix,
    /// Only use the prefix when it is absolutely necessary.
    #[default]
    Never,
}

/// A type that can be used to serialize an attribute.
pub trait SerializeAttributeAccess: Sized {
    /// The type of the value that is returned when serialization is successful.
    type Ok;
    /// The type of the error that is returned when serialization fails.
    type Error: Error;

    /// Set whether to enforce a prefix when serializing.
    fn include_prefix(&mut self, should_include: IncludePrefix) -> Result<Self::Ok, Self::Error>;

    /// Set the preferred prefix to use when serializing.
    fn preferred_prefix(
        &mut self,
        preferred_prefix: Option<Prefix<'_>>,
    ) -> Result<Self::Ok, Self::Error>;

    /// Serialize the attribute.
    fn end<S: AsRef<str>>(self, value: S) -> Result<Self::Ok, Self::Error>;
}

/// A type that can be serialized as an attribute. Since this is a separate trait from [`Serialize`], it is possible to choose between serializing a type as an attribute or as an element.
///
/// To see the documentation for the derive macro, see [`xmlity_derive::SerializeAttribute`].
pub trait SerializeAttribute: Sized {
    /// Serialize the attribute.
    fn serialize_attribute<S: AttributeSerializer>(&self, serializer: S)
        -> Result<S::Ok, S::Error>;
}

/// A trait for serializing sub-elements/sub-attributes of a type. This can be used to more easily include common attributes/elements in multiple types, instead of repeating the same code.
///
/// To see the documentation for the derive macro, see [`xmlity_derive::SerializationGroup`].
pub trait SerializationGroup: Sized {
    /// Serialize the attributes of the type.
    ///
    /// Returns true if the deserializer made progress
    fn serialize_attributes<S: SerializeAttributes>(&self, serializer: S) -> Result<(), S::Error> {
        let _ = serializer;

        Ok(())
    }

    /// Serialize the children of the type.
    ///
    /// Returns true if the deserializer made progress
    fn serialize_children<S: SerializeChildren>(&self, serializer: S) -> Result<(), S::Error> {
        let _ = serializer;

        Ok(())
    }
}
