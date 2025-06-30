use crate::define_serialize_test;
use xmlity::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[xvalue(serialize_with = serialize)]
enum ProcessContentsValue {
    Skip,
    Lax,
    Strict,
}

fn serialize<S>(value: &ProcessContentsValue, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value: String = (*value).into();
    Serialize::serialize(String::as_str(&value.to_string()), serializer)
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

define_serialize_test!(
    serialize_with_test,
    [
        (ProcessContentsValue::Skip, "skip"),
        (ProcessContentsValue::Lax, "lax"),
        (ProcessContentsValue::Strict, "strict")
    ]
);
