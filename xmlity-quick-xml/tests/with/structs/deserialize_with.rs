use crate::define_deserialize_test;
use xmlity::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[xvalue(deserialize_with = deserialize)]
struct ProcessContentsValue(f32);

fn deserialize<'de, D>(deserializer: D) -> Result<ProcessContentsValue, D::Error>
where
    D: Deserializer<'de>,
{
    f32::deserialize(deserializer).map(ProcessContentsValue)
}

define_deserialize_test!(
    deserialize_with_test,
    [
        (ProcessContentsValue(1.0), "1"),
        (ProcessContentsValue(2.5), "2.5")
    ]
);
