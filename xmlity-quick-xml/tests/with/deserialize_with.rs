use core::fmt;
use xmlity::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[xvalue(deserialize_with = deserialize)]
pub enum ProcessContentsValue {
    Skip,
    Lax,
    Strict,
}

pub fn deserialize<'de, D>(
    deserializer: D,
) -> ::core::result::Result<ProcessContentsValue, D::Error>
where
    D: ::xmlity::Deserializer<'de>,
{
    let text: String = ::xmlity::Deserialize::deserialize(deserializer)?;
    let value: String = text.parse().map_err(::xmlity::de::Error::custom)?;
    ProcessContentsValue::try_from(value).map_err(::xmlity::de::Error::custom)
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
    fn try_from(value: String) -> ::core::result::Result<Self, Self::Error> {
        match String::as_str(&value) {
            "skip" => Ok(ProcessContentsValue::Skip),
            "lax" => Ok(ProcessContentsValue::Lax),
            "strict" => Ok(ProcessContentsValue::Strict),
            _ => Err(ProcessContentsValueParseError::NonExistent { value }),
        }
    }
}
