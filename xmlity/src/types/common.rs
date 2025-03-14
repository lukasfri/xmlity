//! This module contains implementations for common types that do not fit into any other module.

use crate::{Deserialize, Deserializer, Serialize, SerializeAttribute};

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Option<T> {
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        Deserialize::deserialize(reader).map(Some)
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn serialize<S: crate::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(value) = self {
            value.serialize(serializer)
        } else {
            serializer.serialize_none()
        }
    }
}

impl<T: SerializeAttribute> SerializeAttribute for Option<T> {
    fn serialize_attribute<S: crate::AttributeSerializer>(
        &self,
        mut serializer: S,
    ) -> Result<S::Ok, S::Error> {
        if let Some(value) = self {
            value.serialize_attribute(serializer)
        } else {
            serializer.serialize_none()
        }
    }
}
