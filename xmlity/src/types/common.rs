//! This module contains implementations for common types that do not fit into any other module.

use crate::{
    de::{self, AttributesAccess, DeserializationGroupBuilder, SeqAccess},
    ser::{SerializeAttributes, SerializeChildren},
    DeserializationGroup, Deserialize, Deserializer, SerializationGroup, Serialize,
    SerializeAttribute,
};

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

/// This builder is used to deserialize optional groups.
pub struct OptionBuilder<T>(T);

impl<'de, T: DeserializationGroupBuilder<'de>> DeserializationGroupBuilder<'de>
    for OptionBuilder<T>
{
    type Value = Option<T::Value>;

    fn contribute_attributes<D: AttributesAccess<'de>>(
        &mut self,
        access: D,
    ) -> Result<bool, D::Error> {
        self.0.contribute_attributes(access)
    }

    fn attributes_done(&self) -> bool {
        self.0.attributes_done()
    }

    fn contribute_elements<D: SeqAccess<'de>>(&mut self, access: D) -> Result<bool, D::Error> {
        self.0.contribute_elements(access)
    }

    fn elements_done(&self) -> bool {
        self.0.elements_done()
    }

    fn finish<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(self.0.finish::<E>().ok())
    }
}

impl<'de, T: DeserializationGroup<'de>> DeserializationGroup<'de> for Option<T> {
    type Builder = OptionBuilder<T::Builder>;

    fn builder() -> Self::Builder {
        OptionBuilder(T::builder())
    }
}

impl<T: SerializationGroup> SerializationGroup for Option<T> {
    fn serialize_attributes<S: SerializeAttributes>(&self, serializer: S) -> Result<(), S::Error> {
        if let Some(value) = self {
            value.serialize_attributes(serializer)
        } else {
            Ok(())
        }
    }

    fn serialize_children<S: SerializeChildren>(&self, serializer: S) -> Result<(), S::Error> {
        if let Some(value) = self {
            value.serialize_children(serializer)
        } else {
            Ok(())
        }
    }
}

impl<T: Serialize> Serialize for Box<T> {
    fn serialize<S: crate::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (**self).serialize(serializer)
    }
}

impl<T: SerializeAttribute> SerializeAttribute for Box<T> {
    fn serialize_attribute<S: crate::AttributeSerializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        (**self).serialize_attribute(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Box<T> {
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        T::deserialize(reader).map(Box::new)
    }
}

impl<T: SerializationGroup> SerializationGroup for Box<T> {
    fn serialize_attributes<S: SerializeAttributes>(&self, serializer: S) -> Result<(), S::Error> {
        (**self).serialize_attributes(serializer)
    }

    fn serialize_children<S: SerializeChildren>(&self, serializer: S) -> Result<(), S::Error> {
        (**self).serialize_children(serializer)
    }
}

impl<'de, T: DeserializationGroup<'de>> DeserializationGroup<'de> for Box<T> {
    type Builder = BoxBuilder<'de, T>;

    fn builder() -> Self::Builder {
        BoxBuilder(T::builder())
    }
}

/// Builder for `Box<T>`.
pub struct BoxBuilder<'de, T: DeserializationGroup<'de>>(T::Builder);

impl<'de, T: DeserializationGroup<'de>> DeserializationGroupBuilder<'de> for BoxBuilder<'de, T> {
    type Value = Box<T>;
    fn contribute_attributes<D: AttributesAccess<'de>>(
        &mut self,
        access: D,
    ) -> Result<bool, D::Error> {
        self.0.contribute_attributes(access)
    }

    fn attributes_done(&self) -> bool {
        self.0.attributes_done()
    }

    fn contribute_elements<D: SeqAccess<'de>>(&mut self, access: D) -> Result<bool, D::Error> {
        self.0.contribute_elements(access)
    }

    fn elements_done(&self) -> bool {
        self.0.elements_done()
    }

    fn finish<E: de::Error>(self) -> Result<Self::Value, E> {
        self.0.finish().map(Box::new)
    }
}
