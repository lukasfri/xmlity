use std::borrow::Cow;

use crate::{Deserialize, Serialize};

// Cow
impl<'a, 'de, T: Deserialize<'de> + Clone> Deserialize<'de> for Cow<'a, T> {
    fn deserialize<D: crate::Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        Deserialize::deserialize(reader).map(Cow::Owned)
    }
}

impl<'a, T: Serialize + Clone> Serialize for Cow<'a, T> {
    fn serialize<S: crate::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        T::serialize(self, serializer)
    }
}

// Explicitly not implementing for Rc and Arc for now.
