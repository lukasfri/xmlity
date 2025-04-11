//! This module contains some utility types and visitors that can be reused.

use core::fmt::{self, Debug};
use std::{marker::PhantomData, str::FromStr};

use crate::{
    de::{self, Visitor, XmlCData, XmlText},
    ser::SerializeSeq,
    types::value::XmlDecl,
    Deserialize, Deserializer, Serialize, Serializer,
};

use super::value::{XmlComment, XmlDoctype, XmlPI};

/// This utility type represents an XML root document.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[non_exhaustive]
pub struct XmlRoot<T> {
    /// The declaration of the XML document.
    pub decl: Option<XmlDecl>,
    pub top: Vec<XmlRootTop<T>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum XmlRootTop<T> {
    /// An element of the XML document.
    Value(T),
    /// A comment in the XML document.
    Comment(XmlComment),
    /// A processing instructions in the XML document.
    PI(XmlPI),
    /// A doctype declarations in the XML document.
    Doctype(XmlDoctype),
}

impl<T> XmlRoot<T> {
    /// Creates a new `XmlRoot` with the given value.
    pub fn new(value: T) -> Self {
        Self {
            decl: None,
            top: Vec::new(),
        }
    }
    /// Adds a comment to the XML document.
    pub fn with_comment<U: Into<XmlComment>>(mut self, comment: U) -> Self {
        self.comments.push(comment.into());
        self
    }

    /// Adds multiple attributes to the element.
    pub fn with_comments<U: Into<XmlComment>, I: IntoIterator<Item = U>>(
        mut self,
        comments: I,
    ) -> Self {
        self.comments.extend(comments.into_iter().map(Into::into));
        self
    }

    /// Adds a processing instruction to the XML document.
    pub fn with_pi<U: Into<XmlPI>>(mut self, pi: U) -> Self {
        self.pis.push(pi.into());
        self
    }

    /// Adds multiple processing instructions to the XML document.
    pub fn with_pis<U: Into<XmlPI>, I: IntoIterator<Item = U>>(mut self, pis: I) -> Self {
        self.pis.extend(pis.into_iter().map(Into::into));
        self
    }

    /// Adds a doctype declaration to the XML document.
    pub fn with_doctype<U: Into<XmlDoctype>>(mut self, doctype: U) -> Self {
        self.doctype.push(doctype.into());
        self
    }

    /// Adds multiple doctype declarations to the XML document.
    pub fn with_doctypes<U: Into<XmlDoctype>, I: IntoIterator<Item = U>>(
        mut self,
        doctypes: I,
    ) -> Self {
        self.doctype.extend(doctypes.into_iter().map(Into::into));
        self
    }

    /// Adds a declaration to the XML document.
    pub fn with_decl<U: Into<XmlDecl>>(mut self, decl: U) -> Self {
        self.decl.replace(decl.into());
        self
    }
}

impl<'de, T: Deserialize<'de> + 'de> Deserialize<'de> for XmlRoot<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct __Visitor<'v, T> {
            marker: ::core::marker::PhantomData<XmlRoot<T>>,
            lifetime: ::core::marker::PhantomData<&'v ()>,
        }

        impl<'v, T: Deserialize<'v> + 'v> crate::de::Visitor<'v> for __Visitor<'v, T> {
            type Value = XmlRoot<T>;

            fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                formatter.write_str("Root element")
            }

            fn visit_seq<S>(self, mut sequence: S) -> Result<Self::Value, S::Error>
            where
                S: crate::de::SeqAccess<'v>,
            {
                let mut decl = None;

                if let Ok(Some(v_decl)) = sequence.next_element_seq::<XmlDecl>() {
                    decl = Some(v_decl);
                }

                let mut top = Vec::new();

                loop {
                    if let Ok(Some(value)) = sequence.next_element_seq::<T>() {
                        top.push(XmlRootTop::Value(value));
                        continue;
                    }

                    if let Ok(Some(comment)) = sequence.next_element_seq::<XmlComment>() {
                        top.push(XmlRootTop::Comment(comment));
                        continue;
                    }
                    if let Ok(Some(pi)) = sequence.next_element_seq::<XmlPI>() {
                        top.push(XmlRootTop::PI(pi));
                        continue;
                    }
                    if let Ok(Some(doctype)) = sequence.next_element_seq::<XmlDoctype>() {
                        top.push(XmlRootTop::Doctype(doctype));
                        continue;
                    }

                    break;
                }

                Ok(XmlRoot { decl, top })
            }
        }

        deserializer.deserialize_seq(__Visitor {
            lifetime: ::core::marker::PhantomData,
            marker: ::core::marker::PhantomData,
        })
    }
}

impl<T> Serialize for XmlRoot<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq()?;

        seq.serialize_element(&self.decl)?;
        seq.serialize_element(&self.top)?;

        seq.end()
    }
}

/// A visitor for deserializing a string from a CDATA section.
pub struct FromCDataVisitor<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Default for FromCDataVisitor<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for FromCDataVisitor<T>
where
    T: FromStr,
{
    type Value = T;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string")
    }

    fn visit_cdata<E, V: XmlCData>(self, v: V) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        v.as_str().parse().map_err(|_| E::custom("invalid value"))
    }
}

/// A wrapper type for CDATA sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CData<S>(pub S);

impl<'de, S: FromStr + Deserialize<'de>> Deserialize<'de> for CData<S> {
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        reader
            .deserialize_any(FromCDataVisitor::default())
            .map(CData)
    }
}

impl<S: AsRef<str>> Serialize for CData<S> {
    fn serialize<Ser: Serializer>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error> {
        serializer.serialize_cdata(self.0.as_ref())
    }
}

/// A type that ignores that uses the value that visits it, but results in nothing. Useful for skipping over values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IgnoredAny;

impl<'de> Deserialize<'de> for IgnoredAny {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct __Visitor<'v> {
            marker: ::core::marker::PhantomData<IgnoredAny>,
            lifetime: ::core::marker::PhantomData<&'v ()>,
        }

        impl<'v> crate::de::Visitor<'v> for __Visitor<'v> {
            type Value = IgnoredAny;

            fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                formatter.write_str("ignored any value")
            }

            fn visit_seq<S>(self, _sequence: S) -> Result<Self::Value, S::Error>
            where
                S: crate::de::SeqAccess<'v>,
            {
                Ok(IgnoredAny)
            }

            fn visit_text<E, V: XmlText>(self, _value: V) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(IgnoredAny)
            }

            fn visit_cdata<E, V: XmlCData>(self, _value: V) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(IgnoredAny)
            }

            fn visit_element<A>(self, _element: A) -> Result<Self::Value, A::Error>
            where
                A: de::ElementAccess<'v>,
            {
                Ok(IgnoredAny)
            }

            fn visit_attribute<A>(self, _attribute: A) -> Result<Self::Value, A::Error>
            where
                A: de::AttributeAccess<'v>,
            {
                Ok(IgnoredAny)
            }

            fn visit_pi<E, V: AsRef<[u8]>>(self, _value: V) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(IgnoredAny)
            }

            fn visit_decl<E, V: AsRef<[u8]>>(
                self,
                _version: V,
                _encoding: Option<V>,
                _standalone: Option<V>,
            ) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(IgnoredAny)
            }

            fn visit_comment<E, V: AsRef<[u8]>>(self, _value: V) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(IgnoredAny)
            }

            fn visit_doctype<E, V: AsRef<[u8]>>(self, _value: V) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(IgnoredAny)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(IgnoredAny)
            }
        }

        deserializer.deserialize_any(__Visitor {
            lifetime: ::core::marker::PhantomData,
            marker: ::core::marker::PhantomData,
        })
    }
}
