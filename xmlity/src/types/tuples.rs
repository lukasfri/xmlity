use crate::{de, ser::SerializeSeq, Deserialize, Deserializer, Serialize};

macro_rules! impl_serialize_tuple {
    (@impl $(($name:ident, $index:tt)),+) => {
        impl<$($name: Serialize),+> Serialize for ($($name,)+) {
            fn serialize<S: crate::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut seq = serializer.serialize_seq()?;
                $(seq.serialize_element(&self.$index)?;)+
                seq.end()
            }
        }
    };
    //Recursive case
    (($first_name:ident, $first_index:tt) $(,($name:ident, $index:tt))*) => {
        impl_serialize_tuple!(@impl ($first_name, $first_index) $(,($name, $index))*);
        impl_serialize_tuple!($(($name, $index)),*);
    };
    //Base case
    () => {};
}

impl_serialize_tuple!(
    (T16, 15),
    (T15, 14),
    (T14, 13),
    (T13, 12),
    (T12, 11),
    (T11, 10),
    (T10, 9),
    (T9, 8),
    (T8, 7),
    (T7, 6),
    (T6, 5),
    (T5, 4),
    (T4, 3),
    (T3, 2),
    (T2, 1),
    (T1, 0)
);

macro_rules! impl_deserialize_tuple {
    (@impl $(($name:ident)),+) => {
        impl<'de, $($name: Deserialize<'de>),*> Deserialize<'de> for ($($name,)*) {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                struct __Visitor<'__visitor, $($name),*> {
                    marker: ::core::marker::PhantomData<($($name,)*)>,
                    lifetime: ::core::marker::PhantomData<&'__visitor ()>,
                }
                impl<'__visitor, $($name: Deserialize<'__visitor>),*> de::Visitor<'__visitor> for __Visitor<'__visitor, $($name),*> {
                    type Value = ($($name,)*);
                    fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(formatter, "tuple")
                    }
                    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
                    where
                        S: crate::de::SeqAccess<'__visitor>,
                    {
                        Ok(($(
                            seq.next_element::<$name>()?.ok_or_else(de::Error::missing_data)?,
                        )*))
                    }
                }
                deserializer.deserialize_seq(__Visitor {
                    marker: ::core::marker::PhantomData,
                    lifetime: ::core::marker::PhantomData,
                })
            }
        }
    };
    //Recursive case
    (($first_name:ident) $(,($name:ident))*) => {
        impl_deserialize_tuple!(@impl ($first_name) $(,($name))*);
        impl_deserialize_tuple!($(($name)),*);
    };
    //Base case
    () => {};
}

impl_deserialize_tuple!(
    (T16),
    (T15),
    (T14),
    (T13),
    (T12),
    (T11),
    (T10),
    (T9),
    (T8),
    (T7),
    (T6),
    (T5),
    (T4),
    (T3),
    (T2),
    (T1)
);
