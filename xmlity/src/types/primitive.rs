use core::fmt;
use std::ops::Deref;

use crate::de::{self, Deserialize, Deserializer};
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

impl_serialize_for_primitive!(
    u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize, f32, f64, char
);

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

impl_deserialize_for_primitive!(
    u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize, f32, f64, char
);

impl Serialize for bool {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_text(if *self { "true" } else { "false" })
    }
}

impl<'de> Deserialize<'de> for bool {
    fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
        pub struct BoolVisitor;

        impl BoolVisitor {
            pub fn from_xml_str(s: &str) -> Option<bool> {
                match s.trim().to_lowercase().as_str() {
                    "true" => Some(true),
                    "false" => Some(false),
                    "1" => Some(true),
                    "0" => Some(false),
                    "yes" => Some(true),
                    "no" => Some(false),
                    _ => None,
                }
            }
        }

        impl<'v> de::Visitor<'v> for BoolVisitor {
            type Value = bool;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a string")
            }

            fn visit_text<E, V>(self, v: V) -> Result<Self::Value, E>
            where
                E: de::Error,
                V: de::XmlText<'v>,
            {
                Self::from_xml_str(v.into_string().deref())
                    .ok_or_else(|| E::custom("invalid value"))
            }
            fn visit_cdata<E, V>(self, v: V) -> Result<Self::Value, E>
            where
                E: de::Error,
                V: de::XmlCData<'v>,
            {
                Self::from_xml_str(v.as_str()).ok_or_else(|| E::custom("invalid value"))
            }
        }

        reader.deserialize_any(BoolVisitor)
    }
}

macro_rules! impl_serialize_for_nonzero_primitive {
  ($(($t:ty, $u:ty)),*) => {
      $(
          impl Serialize for $t {
              fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                  serializer.serialize_text(self.get().to_string())
              }
          }
      )*
  };
}

impl_serialize_for_nonzero_primitive!(
    (core::num::NonZeroU8, u8),
    (core::num::NonZeroU16, u16),
    (core::num::NonZeroU32, u32),
    (core::num::NonZeroU64, u64),
    (core::num::NonZeroU128, u128),
    (core::num::NonZeroI8, i8),
    (core::num::NonZeroI16, i16),
    (core::num::NonZeroI32, i32),
    (core::num::NonZeroI64, i64),
    (core::num::NonZeroI128, i128)
);

macro_rules! impl_deserialize_for_nonzero_primitive {
  ($(($t:ty, $u:ty)),*) => {
      $(
            impl<'de> Deserialize<'de> for $t {
                fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
                    let value = <$u as Deserialize<'de>>::deserialize(reader)?;
                    core::num::NonZero::new(value).ok_or_else(|| de::Error::custom("value cannot be zero"))
                }
            }
      )*
  };
}

impl_deserialize_for_nonzero_primitive!(
    (core::num::NonZeroU8, u8),
    (core::num::NonZeroU16, u16),
    (core::num::NonZeroU32, u32),
    (core::num::NonZeroU64, u64),
    (core::num::NonZeroU128, u128),
    (core::num::NonZeroI8, i8),
    (core::num::NonZeroI16, i16),
    (core::num::NonZeroI32, i32),
    (core::num::NonZeroI64, i64),
    (core::num::NonZeroI128, i128)
);
