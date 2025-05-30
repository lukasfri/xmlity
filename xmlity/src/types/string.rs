//! This module contains visitors and [`Serialize`]/[`Deserialize`] implementations for strings and string-like types.

use super::utils::FromCDataVisitor;
use core::fmt;
use core::fmt::Debug;
use std::{marker::PhantomData, str::FromStr};

use crate::{
    de::{Error, Visitor, XmlCData, XmlText},
    Deserialize, Deserializer, Serialize, Serializer,
};

/// This visitor allows for deserializing a string from a trimmed text node.
pub struct FromTrimmedTextVisitor<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Default for FromTrimmedTextVisitor<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<'de, T> Visitor<'de> for FromTrimmedTextVisitor<T>
where
    T: FromStr,
{
    type Value = Trim<T>;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string")
    }
    fn visit_text<E, V>(self, v: V) -> Result<Self::Value, E>
    where
        E: Error,
        V: XmlText<'de>,
    {
        v.into_string()
            .trim()
            .parse()
            .map(Trim)
            .map_err(|_| E::custom("invalid value"))
    }
}

/// This type allows for deserializing a string from a trimmed text node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Trim<T>(pub T);

impl<'de, T: FromStr> Deserialize<'de> for Trim<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(FromTrimmedTextVisitor::default())
    }
}

impl<T: AsRef<str>> Serialize for Trim<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.as_ref().serialize(serializer)
    }
}

/// This visitor allows for deserializing a string from a text node using [`std::str::FromStr::from_str`].
pub struct FromTextVisitor<T: FromStr> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: FromStr> Default for FromTextVisitor<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for FromTextVisitor<T>
where
    T: FromStr,
{
    type Value = T;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string")
    }

    fn visit_text<E, V>(self, v: V) -> Result<Self::Value, E>
    where
        E: Error,
        V: XmlText<'de>,
    {
        v.into_string()
            .parse()
            .map_err(|_| E::custom("invalid value"))
    }
}

/// This visitor allows for deserializing a string from a text node or CDATA section, using [`std::str::FromStr::from_str`].
pub struct FromStrVisitor<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> Default for FromStrVisitor<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for FromStrVisitor<T>
where
    T: FromStr,
{
    type Value = T;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string")
    }

    fn visit_text<E, V>(self, v: V) -> Result<Self::Value, E>
    where
        E: Error,
        V: XmlText<'de>,
    {
        FromTextVisitor::default().visit_text(v)
    }
    fn visit_cdata<E, V>(self, v: V) -> Result<Self::Value, E>
    where
        E: Error,
        V: XmlCData<'de>,
    {
        FromCDataVisitor::default().visit_cdata(v)
    }
}

impl<'de> Deserialize<'de> for String {
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        reader.deserialize_any(FromStrVisitor::default())
    }
}

impl Serialize for String {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_str().serialize(serializer)
    }
}

impl Serialize for &str {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        //TODO: Change to serialize as CDATA if it contains invalid XML characters
        serializer.serialize_text(self)
    }
}

impl Serialize for str {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        //TODO: Change to serialize as CDATA if it contains invalid XML characters
        serializer.serialize_text(self)
    }
}
