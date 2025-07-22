//! This module contains visitors and [`Serialize`]/[`Deserialize`] implementations for iterators and common collections.
use std::collections::{BTreeSet, HashMap, HashSet, LinkedList, VecDeque};
use std::marker::PhantomData;

use crate::de::{self, Visitor};
use crate::ser::SerializeSeq;
use crate::{de::SeqAccess, Deserialize, Deserializer};
use crate::{Serialize, Serializer};
use core::fmt;
use std::iter::FromIterator;

use super::utils::ValueOrWhitespace;

/// This trait is used to decide how [`IteratorVisitor`] will deserialize a type.
pub trait IteratorVisitorMiddleware<T> {
    /// The output of the middleware.
    type Output;

    /// Takes in the item iterator and results in the [`IteratorVisitorMiddleware::Output`] type.
    fn transform<I>(iter: I) -> Self::Output
    where
        I: IntoIterator<Item = T>;
}

impl<T, F: FromIterator<T>> IteratorVisitorMiddleware<T> for F {
    type Output = F;
    fn transform<I>(iter: I) -> Self::Output
    where
        I: IntoIterator<Item = T>,
    {
        F::from_iter(iter)
    }
}

struct NoWhitespaceIter<T, O> {
    _marker: PhantomData<(T, O)>,
    result: O,
}

impl<'de, T: Deserialize<'de>, O: FromIterator<T>> FromIterator<ValueOrWhitespace<'de, T>>
    for NoWhitespaceIter<T, O>
{
    fn from_iter<I: IntoIterator<Item = ValueOrWhitespace<'de, T>>>(iter: I) -> Self {
        Self {
            _marker: PhantomData,
            result: O::from_iter(iter.into_iter().filter_map(|a| match a {
                ValueOrWhitespace::Whitespace(_) => None,
                ValueOrWhitespace::Value(a) => Some(a),
            })),
        }
    }
}

impl<'de, T: Deserialize<'de>, O: FromIterator<T>> Deserialize<'de> for NoWhitespaceIter<T, O> {
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        reader.deserialize_seq(IteratorVisitor::<_, Self>::default())
    }
}

/// This visitor allows for deserializing an iterator of elements, which can be useful for deserializing sequences of elements into a collection/single value.
pub struct IteratorVisitor<T, M> {
    _marker: PhantomData<(T, M)>,
}

impl<T, M: IteratorVisitorMiddleware<T>> IteratorVisitor<T, M> {
    /// Creates a new [`IteratorVisitor`].
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T, M: IteratorVisitorMiddleware<T>> Default for IteratorVisitor<T, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'de, T, M> Visitor<'de> for IteratorVisitor<T, M>
where
    T: Deserialize<'de>,
    M: IteratorVisitorMiddleware<T>,
    M::Output: de::Deserialize<'de>,
{
    type Value = M::Output;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a sequence of elements")
    }

    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
    where
        S: SeqAccess<'de>,
    {
        Ok(M::transform(std::iter::from_fn(|| {
            seq.next_element_seq::<Option<T>>().ok().flatten().flatten()
        })))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(M::transform(std::iter::empty()))
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
        struct LimitFromIter<const N: usize, T, O> {
            _marker: PhantomData<(T, O)>,
            result: O,
        }

        impl<'de, const N: usize, T: Deserialize<'de>, O: FromIterator<T>> FromIterator<T>
            for LimitFromIter<N, T, O>
        {
            fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
                Self {
                    _marker: PhantomData,
                    result: O::from_iter(iter.into_iter().take(N)),
                }
            }
        }

        impl<'de, const N: usize, T: Deserialize<'de>, O: FromIterator<T>> Deserialize<'de>
            for LimitFromIter<N, T, O>
        {
            fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
                reader.deserialize_seq(IteratorVisitor::<_, Self>::default())
            }
        }

        let vec = reader.deserialize_seq(IteratorVisitor::<
            ValueOrWhitespace<T>,
            NoWhitespaceIter<T, LimitFromIter<N, T, Vec<T>>>,
        >::new())?;

        vec.result
            .result
            .try_into()
            .map_err(|_| de::Error::missing_data())
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
        reader
            .deserialize_seq(IteratorVisitor::<_, NoWhitespaceIter<_, Self>>::default())
            .map(|a| a.result)
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
        reader
            .deserialize_seq(IteratorVisitor::<_, NoWhitespaceIter<_, Self>>::default())
            .map(|a| a.result)
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
        reader
            .deserialize_seq(IteratorVisitor::<_, NoWhitespaceIter<_, Self>>::default())
            .map(|a| a.result)
    }
}

impl<T: Serialize> Serialize for LinkedList<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_seq(self.iter(), serializer)
    }
}

// HashSet
impl<'de, T: Deserialize<'de> + Eq + std::hash::Hash> Deserialize<'de> for HashSet<T> {
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        reader
            .deserialize_seq(IteratorVisitor::<_, NoWhitespaceIter<_, Self>>::default())
            .map(|a| a.result)
    }
}

impl<T: Serialize> Serialize for HashSet<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_seq(self.iter(), serializer)
    }
}

// BTreeSet
impl<'de, T: Deserialize<'de> + Ord> Deserialize<'de> for BTreeSet<T> {
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        reader
            .deserialize_seq(IteratorVisitor::<_, NoWhitespaceIter<_, Self>>::default())
            .map(|a| a.result)
    }
}

impl<T: Serialize> Serialize for BTreeSet<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_seq(self.iter(), serializer)
    }
}

// HashMap
impl<'de, K: Deserialize<'de> + Eq + std::hash::Hash, V: Deserialize<'de>> Deserialize<'de>
    for HashMap<K, V>
{
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        reader
            .deserialize_seq(IteratorVisitor::<_, NoWhitespaceIter<_, Self>>::default())
            .map(|a| a.result)
    }
}

impl<K: Serialize, V: Serialize> Serialize for HashMap<K, V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_seq(self.iter(), serializer)
    }
}
