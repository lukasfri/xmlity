use crate::de::{Deserialize, Deserializer};
use crate::ser::{Serialize, Serializer};
use crate::types::string::FromStrVisitor;

macro_rules! impl_serialize_for_primitive {
  ($($t:ty),*) => {
      $(
          impl Serialize for $t {
              fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                  serializer.serialize_text(self.to_string())
              }
          }
      )*
  };
}

impl_serialize_for_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

macro_rules! impl_deserialize_for_primitive {
  ($($t:ty),*) => {
      $(
          impl<'de> Deserialize<'de> for $t {
              fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
                  reader.deserialize_any(FromStrVisitor::default())
              }
          }
      )*
  };
}

impl_deserialize_for_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
