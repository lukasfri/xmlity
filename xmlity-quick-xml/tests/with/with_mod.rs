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
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> ::core::result::Result<super::ProcessContentsValue, D::Error>
    where
        D: ::xmlity::Deserializer<'de>,
    {
        let text: String = ::xmlity::Deserialize::deserialize(deserializer)?;
        let value: String = text.parse().map_err(::xmlity::de::Error::custom)?;
        super::ProcessContentsValue::try_from(value).map_err(::xmlity::de::Error::custom)
    }
    pub fn serialize<S>(
        value: &super::ProcessContentsValue,
        serializer: S,
    ) -> ::core::result::Result<S::Ok, S::Error>
    where
        S: ::xmlity::Serializer,
    {
        let value: String = (*value).into();
        ::xmlity::Serialize::serialize(
            String::as_str(&::std::string::ToString::to_string(&value)),
            serializer,
        )
    }
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
impl From<ProcessContentsValue> for String {
    fn from(value: ProcessContentsValue) -> Self {
        match value {
            ProcessContentsValue::Skip => String::from("skip"),
            ProcessContentsValue::Lax => String::from("lax"),
            ProcessContentsValue::Strict => String::from("strict"),
        }
    }
}
