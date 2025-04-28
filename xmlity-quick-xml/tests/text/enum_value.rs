use crate::{define_test, utils::quick_xml_deserialize_test};

use rstest::rstest;
use xmlity::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Union {
    #[xvalue(value = "restriction")]
    Restriction,
    #[xvalue(value = "extension")]
    Extension,
}

define_test!(
    union_test,
    [
        (Union::Restriction, "restriction"),
        (Union::Extension, "extension")
    ]
);

#[rstest]
#[case::restriction("Restriction")]
#[case::extension("Extension")]
fn wrong_union_test(#[case] xml: &str) {
    let union: Result<Union, _> = quick_xml_deserialize_test(xml);

    assert!(union.is_err());
}
