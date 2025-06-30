use crate::define_serialize_test;
use xmlity::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[xvalue(serialize_with = serialize)]
struct ProcessContentsValue(f32);

fn serialize<S>(value: &ProcessContentsValue, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    Serialize::serialize(&value.0, serializer)
}

define_serialize_test!(
    serialize_with_test,
    [
        (ProcessContentsValue(1.0), "1"),
        (ProcessContentsValue(2.5), "2.5")
    ]
);
