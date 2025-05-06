//! This module contains visitors and [`Serialize`]/[`Deserialize`] implementations for iterators and common collections.
use std::collections::{BTreeSet, HashMap, HashSet, LinkedList, VecDeque};
use std::marker::PhantomData;

use crate::de::{self, Visitor};
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

fn serialize_seq<T, S>(iter: T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: IntoIterator,
    T::Item: Serialize,
    S: Serializer,
{
    let mut seq = serializer.serialize_seq()?;
    for element in iter {
        seq.serialize_element(&element)?;
    }
    seq.end()
}

/// This visitor allows for deserializing an iterator of elements up to a certain limit, which can be useful for deserializing sequences of elements into an array/fixed-size list.
///
/// For now, this visitor is pub(crate) because it's yet to be decided whether it could be replaced by a more general solution.
pub(crate) struct LimitIteratorVisitor<T, V: FromIterator<T>> {
    _marker: PhantomData<(T, V)>,
    limit: usize,
}

impl<T, V: FromIterator<T>> LimitIteratorVisitor<T, V> {
    /// Creates a new [`IteratorVisitor`].
    pub fn new(limit: usize) -> Self {
        Self {
            _marker: PhantomData,
            limit,
        }
    }
}

impl<'de, T, V> Visitor<'de> for LimitIteratorVisitor<T, V>
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
        Ok(
            std::iter::from_fn(|| seq.next_element_seq::<T>().ok().flatten())
                .take(self.limit)
                .collect::<V>(),
        )
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

// Array
impl<'de, const N: usize, T: Deserialize<'de>> Deserialize<'de> for [T; N] {
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        let vec: Vec<T> = reader.deserialize_seq(LimitIteratorVisitor::new(N))?;

        vec.try_into().map_err(|_| de::Error::missing_data())
    }
}

impl<const N: usize, T: Serialize> Serialize for [T; N] {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_slice().serialize(serializer)
    }
}

// Vec
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Vec<T> {
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        reader.deserialize_seq(IteratorVisitor::new())
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_slice().serialize(serializer)
    }
}

// VecDeque
impl<'de, T: Deserialize<'de>> Deserialize<'de> for VecDeque<T> {
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        reader.deserialize_seq(IteratorVisitor::new())
    }
}

impl<T: Serialize> Serialize for VecDeque<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_slices().0.serialize(serializer)
    }
}

// LinkedList
impl<'de, T: Deserialize<'de>> Deserialize<'de> for LinkedList<T> {
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        reader.deserialize_seq(IteratorVisitor::new())
    }
}

impl<T: Serialize> Serialize for LinkedList<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.iter().collect::<Vec<_>>().serialize(serializer)
    }
}

// HashSet
impl<'de, T: Deserialize<'de> + Eq + std::hash::Hash> Deserialize<'de> for HashSet<T> {
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        reader.deserialize_seq(IteratorVisitor::new())
    }
}

impl<T: Serialize> Serialize for HashSet<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.iter().collect::<Vec<_>>().serialize(serializer)
    }
}

// BTreeSet
impl<'de, T: Deserialize<'de> + Ord> Deserialize<'de> for BTreeSet<T> {
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        reader.deserialize_seq(IteratorVisitor::new())
    }
}

impl<T: Serialize> Serialize for BTreeSet<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.iter().collect::<Vec<_>>().serialize(serializer)
    }
}

// HashMap
impl<'de, K: Deserialize<'de> + Eq + std::hash::Hash, V: Deserialize<'de>> Deserialize<'de>
    for HashMap<K, V>
{
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        reader.deserialize_seq(IteratorVisitor::<(K, V), _>::new())
    }
}

impl<K: Serialize, V: Serialize> Serialize for HashMap<K, V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_seq(self.iter(), serializer)
    }
}
