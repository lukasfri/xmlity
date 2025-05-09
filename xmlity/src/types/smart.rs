use std::borrow::Cow;

use crate::{Deserialize, Serialize};

// Cow
impl<'de, T: Deserialize<'de> + Clone> Deserialize<'de> for Cow<'_, T> {
    fn deserialize<D: crate::Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        Deserialize::deserialize(reader).map(Cow::Owned)
    }
}

impl<T: Serialize + Clone> Serialize for Cow<'_, T> {
    fn serialize<S: crate::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        T::serialize(self, serializer)
    }
}

// Explicitly not implementing for Rc and Arc for now.
