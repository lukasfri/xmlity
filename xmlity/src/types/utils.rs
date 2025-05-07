//! This module contains some utility types and visitors that can be reused.

use core::fmt::{self, Debug};
use std::{marker::PhantomData, str::FromStr};

use crate::{
    de::{
        self, Visitor, XmlCData, XmlComment, XmlDeclaration, XmlDoctype, XmlProcessingInstruction,
        XmlText,
    },
    types::value::XmlDecl,
    Deserialize, Deserializer, Serialize, Serializer,
};

use super::value::{self};

/// This utility type represents an XML root document.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct XmlRoot<T> {
    /// The declaration of the XML document.
    pub decl: Option<XmlDecl>,
    /// The top-level elements of the XML document.
    pub elements: Vec<XmlRootTop<T>>,
}

impl<T: Serialize> crate::Serialize for XmlRoot<T> {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<<S as crate::Serializer>::Ok, <S as crate::Serializer>::Error>
    where
        S: crate::Serializer,
    {
        let mut __elements = crate::Serializer::serialize_seq(serializer)?;
        crate::ser::SerializeSeq::serialize_element(&mut __elements, &self.decl)?;
        crate::ser::SerializeSeq::serialize_element(&mut __elements, &self.elements)?;
        crate::ser::SerializeSeq::end(__elements)
    }
}

impl<'__deserialize, T: Deserialize<'__deserialize> + Debug> Deserialize<'__deserialize>
    for XmlRoot<T>
{
    fn deserialize<D>(__deserializer: D) -> Result<Self, <D as Deserializer<'__deserialize>>::Error>
    where
        D: Deserializer<'__deserialize>,
    {
        struct __XmlRootVisitor<'__visitor, T> {
            marker: ::core::marker::PhantomData<XmlRoot<T>>,
            lifetime: ::core::marker::PhantomData<&'__visitor ()>,
        }
        impl<'__visitor, T: Deserialize<'__visitor> + Debug> crate::de::Visitor<'__visitor>
            for __XmlRootVisitor<'__visitor, T>
        {
            type Value = XmlRoot<T>;
            fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(formatter, "struct XmlRoot")
            }

            fn visit_seq<S>(self, mut sequence: S) -> Result<Self::Value, S::Error>
            where
                S: de::SeqAccess<'__visitor>,
            {
                Ok(Self::Value {
                    decl: crate::de::SeqAccess::next_element::<XmlDecl>(&mut sequence)
                        .ok()
                        .flatten(),
                    elements: crate::de::SeqAccess::next_element_seq::<Vec<XmlRootTop<T>>>(
                        &mut sequence,
                    )?
                    .unwrap_or_default(),
                })
            }
        }
        Deserializer::deserialize_seq(
            __deserializer,
            __XmlRootVisitor {
                lifetime: ::core::marker::PhantomData,
                marker: ::core::marker::PhantomData,
            },
        )
    }
}

/// A top-level element of the XML document.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum XmlRootTop<T> {
    /// An element of the XML document.
    Value(T),
    /// A comment in the XML document.
    Comment(value::XmlComment),
    /// A processing instructions in the XML document.
    PI(value::XmlProcessingInstruction),
    /// A doctype declarations in the XML document.
    Doctype(value::XmlDoctype),
}

impl<T> From<value::XmlComment> for XmlRootTop<T> {
    fn from(value: value::XmlComment) -> Self {
        XmlRootTop::Comment(value)
    }
}

impl<T> From<value::XmlProcessingInstruction> for XmlRootTop<T> {
    fn from(value: value::XmlProcessingInstruction) -> Self {
        XmlRootTop::PI(value)
    }
}

impl<T> From<value::XmlDoctype> for XmlRootTop<T> {
    fn from(value: value::XmlDoctype) -> Self {
        XmlRootTop::Doctype(value)
    }
}

impl<T: Serialize> crate::Serialize for XmlRootTop<T> {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<<S as crate::Serializer>::Ok, <S as crate::Serializer>::Error>
    where
        S: crate::Serializer,
    {
        match self {
            XmlRootTop::Value(__v) => crate::Serialize::serialize(&__v, serializer),
            XmlRootTop::Comment(__v) => crate::Serialize::serialize(&__v, serializer),
            XmlRootTop::PI(__v) => crate::Serialize::serialize(&__v, serializer),
            XmlRootTop::Doctype(__v) => crate::Serialize::serialize(&__v, serializer),
        }
    }
}

impl<'__deserialize, T: Deserialize<'__deserialize>> Deserialize<'__deserialize> for XmlRootTop<T> {
    fn deserialize<D>(__deserializer: D) -> Result<Self, <D as Deserializer<'__deserialize>>::Error>
    where
        D: Deserializer<'__deserialize>,
    {
        struct __XmlRootTopVisitor<'__visitor, T> {
            marker: ::core::marker::PhantomData<XmlRootTop<T>>,
            lifetime: ::core::marker::PhantomData<&'__visitor ()>,
        }
        impl<'__visitor, T: Deserialize<'__visitor>> crate::de::Visitor<'__visitor>
            for __XmlRootTopVisitor<'__visitor, T>
        {
            type Value = XmlRootTop<T>;
            fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(formatter, "enum XmlRootTop")
            }
            fn visit_seq<S>(
                self,
                mut __sequence: S,
            ) -> Result<Self::Value, <S as crate::de::SeqAccess<'__visitor>>::Error>
            where
                S: crate::de::SeqAccess<'__visitor>,
            {
                if let ::core::result::Result::Ok(::core::option::Option::Some(_v)) =
                    crate::de::SeqAccess::next_element::<T>(&mut __sequence)
                {
                    return ::core::result::Result::Ok(XmlRootTop::Value(_v));
                }
                if let ::core::result::Result::Ok(::core::option::Option::Some(_v)) =
                    crate::de::SeqAccess::next_element::<value::XmlComment>(&mut __sequence)
                {
                    return ::core::result::Result::Ok(XmlRootTop::Comment(_v));
                }
                if let ::core::result::Result::Ok(::core::option::Option::Some(_v)) =
                    crate::de::SeqAccess::next_element::<value::XmlProcessingInstruction>(
                        &mut __sequence,
                    )
                {
                    return ::core::result::Result::Ok(XmlRootTop::PI(_v));
                }
                if let ::core::result::Result::Ok(::core::option::Option::Some(_v)) =
                    crate::de::SeqAccess::next_element::<value::XmlDoctype>(&mut __sequence)
                {
                    return ::core::result::Result::Ok(XmlRootTop::Doctype(_v));
                }
                ::core::result::Result::Err(crate::de::Error::no_possible_variant("XmlRootTop"))
            }
        }
        Deserializer::deserialize_seq(
            __deserializer,
            __XmlRootTopVisitor {
                lifetime: ::core::marker::PhantomData,
                marker: ::core::marker::PhantomData,
            },
        )
    }
}

impl<T> XmlRoot<T> {
    /// Creates a new `XmlRoot` with the given value.
    pub fn new() -> Self {
        Self {
            decl: None,
            elements: Vec::new(),
        }
    }

    /// Adds a declaration to the XML document.
    pub fn with_decl<U: Into<XmlDecl>>(mut self, decl: U) -> Self {
        let decl: XmlDecl = decl.into();
        self.decl = Some(decl);
        self
    }

    /// Adds an element to the XML document.
    pub fn with_element<U: Into<T>>(mut self, element: U) -> Self {
        let element: T = element.into();
        self.elements.push(XmlRootTop::Value(element));
        self
    }

    /// Adds multiple elements to the XML document.
    pub fn with_elements<U: Into<T>, I: IntoIterator<Item = U>>(mut self, elements: I) -> Self {
        self.elements.extend(
            elements
                .into_iter()
                .map(Into::<T>::into)
                .map(XmlRootTop::Value),
        );
        self
    }

    /// Adds a comment to the XML document.
    pub fn with_comment<U: Into<value::XmlComment>>(mut self, comment: U) -> Self {
        let comment: value::XmlComment = comment.into();
        self.elements.push(comment.into());
        self
    }

    /// Adds multiple attributes to the element.
    pub fn with_comments<U: Into<value::XmlComment>, I: IntoIterator<Item = U>>(
        mut self,
        comments: I,
    ) -> Self {
        self.elements.extend(
            comments
                .into_iter()
                .map(Into::<value::XmlComment>::into)
                .map(Into::into),
        );
        self
    }

    /// Adds a processing instruction to the XML document.
    pub fn with_pi<U: Into<value::XmlProcessingInstruction>>(mut self, pi: U) -> Self {
        let pi: value::XmlProcessingInstruction = pi.into();
        self.elements.push(pi.into());
        self
    }

    /// Adds multiple processing instructions to the XML document.
    pub fn with_pis<U: Into<value::XmlProcessingInstruction>, I: IntoIterator<Item = U>>(
        mut self,
        pis: I,
    ) -> Self {
        self.elements.extend(
            pis.into_iter()
                .map(Into::<value::XmlProcessingInstruction>::into)
                .map(Into::into),
        );
        self
    }

    /// Adds a doctype declaration to the XML document.
    pub fn with_doctype<U: Into<value::XmlDoctype>>(mut self, doctype: U) -> Self {
        let doctype: value::XmlDoctype = doctype.into();
        self.elements.push(doctype.into());
        self
    }

    /// Adds multiple doctype declarations to the XML document.
    pub fn with_doctypes<U: Into<value::XmlDoctype>, I: IntoIterator<Item = U>>(
        mut self,
        doctypes: I,
    ) -> Self {
        self.elements.extend(
            doctypes
                .into_iter()
                .map(Into::<value::XmlDoctype>::into)
                .map(Into::into),
        );
        self
    }
}

impl<T> Default for XmlRoot<T> {
    fn default() -> Self {
        Self::new()
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
pub struct IgnoreWhitespace;

impl<'de> Deserialize<'de> for IgnoreWhitespace {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct __Visitor<'v> {
            marker: ::core::marker::PhantomData<IgnoreWhitespace>,
            lifetime: ::core::marker::PhantomData<&'v ()>,
        }

        impl<'v> crate::de::Visitor<'v> for __Visitor<'v> {
            type Value = IgnoreWhitespace;

            fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                formatter.write_str("ignored any value")
            }

            fn visit_text<E, V: XmlText>(self, text: V) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if text.as_str().trim().is_empty() {
                    Ok(IgnoreWhitespace)
                } else {
                    Err(E::custom("expected whitespace"))
                }
            }
        }

        deserializer.deserialize_any(__Visitor {
            lifetime: ::core::marker::PhantomData,
            marker: ::core::marker::PhantomData,
        })
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

            fn visit_pi<E, V: XmlProcessingInstruction>(self, _pi: V) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(IgnoredAny)
            }

            fn visit_decl<E, V: XmlDeclaration>(self, _declaration: V) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(IgnoredAny)
            }

            fn visit_comment<E, V: XmlComment>(self, _comment: V) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(IgnoredAny)
            }

            fn visit_doctype<E, V: XmlDoctype>(self, _doctype: V) -> Result<Self::Value, E>
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
