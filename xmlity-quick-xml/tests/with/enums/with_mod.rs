use crate::define_test;
use core::fmt;
use xmlity::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[xvalue(with = with_mod)]
pub enum ProcessContentsValue {
    Skip,
    Lax,
    Strict,
}

pub mod with_mod {
    use super::ProcessContentsValue;
    use xmlity::{de, Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ProcessContentsValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text: String = Deserialize::deserialize(deserializer)?;
        let value: String = text.parse().map_err(de::Error::custom)?;
        ProcessContentsValue::try_from(value).map_err(de::Error::custom)
    }
    pub fn serialize<S>(value: &ProcessContentsValue, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Serialize::serialize(&String::from(*value), serializer)
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum ProcessContentsValueParseError {
    NonExistent { value: String },
}

impl fmt::Display for ProcessContentsValueParseError {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
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
impl From<ProcessContentsValue> for String {
    fn from(value: ProcessContentsValue) -> Self {
        match value {
            ProcessContentsValue::Skip => String::from("skip"),
            ProcessContentsValue::Lax => String::from("lax"),
            ProcessContentsValue::Strict => String::from("strict"),
        }
    }
}

define_test!(
    deserialize_with_test,
    [
        (ProcessContentsValue::Skip, "skip"),
        (ProcessContentsValue::Lax, "lax"),
        (ProcessContentsValue::Strict, "strict")
    ]
);
