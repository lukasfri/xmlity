use core::fmt;
use xmlity::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[xvalue(serialize_with = serialize)]
pub enum ProcessContentsValue {
    Skip,
    Lax,
    Strict,
}

pub fn serialize<S>(
    value: &ProcessContentsValue,
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

#[derive(Debug, PartialEq, Clone)]
pub enum ProcessContentsValueParseError {
    NonExistent { value: String },
}

impl fmt::Display for ProcessContentsValueParseError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
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
