use pretty_assertions::assert_eq;

mod common;
use common::quick_xml_deserialize_test;

use rstest::rstest;
use xmlity::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Union {
    #[xvalue(value = "restriction")]
    Restriction,
    #[xvalue(value = "extension")]
    Extension,
}

#[rstest]
#[case::restriction("restriction", Union::Restriction)]
#[case::extension("extension", Union::Extension)]
fn union_test(#[case] xml: &str, #[case] expected: Union) {
    let union: Result<Union, _> = quick_xml_deserialize_test(xml);

    assert!(union.is_ok(), "Successfully deserialized \"restriction\")");
    let union = union.unwrap();
    assert_eq!(union, expected);
}

#[rstest]
#[case::restriction("Restriction")]
#[case::extension("Extension")]
fn wrong_union_test(#[case] xml: &str) {
    let union: Result<Union, _> = quick_xml_deserialize_test(xml);

    assert!(union.is_err());
}
