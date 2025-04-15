//! This module contains visitors and [`Serialize`]/[`Deserialize`] implementations for iterators and common collections.
use std::marker::PhantomData;

use crate::de::Visitor;
use crate::ser::SerializeSeq;
use crate::{de::SeqAccess, Deserialize, Deserializer};
use crate::{Serialize, Serializer};
use core::fmt;
use std::iter::FromIterator;

/// This visitor allows for deserializing an iterator of elements, which can be useful for deserializing sequences of elements into a collection/single value.
pub struct IteratorVisitor<T, V: FromIterator<T>> {
    _marker: PhantomData<(T, V)>,
}

impl<T, V: FromIterator<T>> IteratorVisitor<T, V> {
    /// Creates a new [`IteratorVisitor`].
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T, V: FromIterator<T>> Default for IteratorVisitor<T, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'de, T, V> Visitor<'de> for IteratorVisitor<T, V>
where
    T: Deserialize<'de>,
    V: FromIterator<T> + Deserialize<'de>,
{
    type Value = V;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a sequence of elements")
    }

    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
    where
        S: SeqAccess<'de>,
    {
        Ok(std::iter::from_fn(|| seq.next_element_seq::<T>().ok().flatten()).collect::<V>())
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Vec<T> {
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        reader.deserialize_any(IteratorVisitor::new())
    }

    fn deserialize_seq<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        reader.deserialize_seq(IteratorVisitor::new())
    }
}

impl<T: Serialize> Serialize for &[T] {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq()?;
        for item in self.iter() {
            seq.serialize_element(item)?;
        }

        seq.end()
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_slice().serialize(serializer)
    }
}
