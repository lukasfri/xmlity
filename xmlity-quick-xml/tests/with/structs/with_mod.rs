use crate::define_test;
use xmlity::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[xvalue(with = with_mod)]
struct ProcessContentsValue(f32);

mod with_mod {
    use super::ProcessContentsValue;
    use xmlity::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ProcessContentsValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        f32::deserialize(deserializer).map(ProcessContentsValue)
    }
    pub fn serialize<S>(value: &ProcessContentsValue, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(&value.0, serializer)
    }
}
define_test!(
    deserialize_with_test,
    [
        (ProcessContentsValue(1.0), "1"),
        (ProcessContentsValue(2.5), "2.5")
    ]
);
