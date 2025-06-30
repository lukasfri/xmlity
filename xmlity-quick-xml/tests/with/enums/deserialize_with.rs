use crate::define_deserialize_test;
use core::fmt;
use xmlity::{de, Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[xvalue(deserialize_with = deserialize)]
pub enum ProcessContentsValue {
    Skip,
    Lax,
    Strict,
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<ProcessContentsValue, D::Error>
where
    D: Deserializer<'de>,
{
    let text: String = Deserialize::deserialize(deserializer)?;
    let value: String = text.parse().map_err(de::Error::custom)?;
    ProcessContentsValue::try_from(value).map_err(de::Error::custom)
}

#[derive(Debug, PartialEq, Clone)]
pub enum ProcessContentsValueParseError {
    NonExistent { value: String },
}

impl fmt::Display for ProcessContentsValueParseError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
impl TryFrom<String> for ProcessContentsValue {
    type Error = ProcessContentsValueParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match String::as_str(&value) {
            "skip" => Ok(ProcessContentsValue::Skip),
            "lax" => Ok(ProcessContentsValue::Lax),
            "strict" => Ok(ProcessContentsValue::Strict),
            _ => Err(ProcessContentsValueParseError::NonExistent { value }),
        }
    }
}

define_deserialize_test!(
    deserialize_with_test,
    [
        (ProcessContentsValue::Skip, "skip"),
        (ProcessContentsValue::Lax, "lax"),
        (ProcessContentsValue::Strict, "strict")
    ]
);
