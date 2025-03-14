use pretty_assertions::assert_eq;

mod common;
use common::{clean_string, quick_xml_deserialize_test, quick_xml_serialize_test};

use xmlity_derive::{Deserialize, Serialize};

const SIMPLE_1D_STRUCT_TEST_XML: &str = "Alpha";

const SIMPLE_WRONG_1D_STRUCT_TEST_XML: &str = "Lmao";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue]
enum EnumValue {
    Alpha,
    Beta,
    Gamma,
}

#[test]
fn simple_1d_struct_serialize() {
    let actual = quick_xml_serialize_test(EnumValue::Alpha).unwrap();

    assert_eq!(actual, clean_string(SIMPLE_1D_STRUCT_TEST_XML));
}

#[test]
fn simple_1d_struct_deserialize() {
    let actual: EnumValue =
        quick_xml_deserialize_test(clean_string(SIMPLE_1D_STRUCT_TEST_XML).as_str()).unwrap();

    let expected = EnumValue::Alpha;

    assert_eq!(actual, expected);
}

#[test]
fn simple_wrong_1d_struct_deserialize() {
    let actual: Result<EnumValue, _> =
        quick_xml_deserialize_test(clean_string(SIMPLE_WRONG_1D_STRUCT_TEST_XML).as_str());
    assert!(actual.is_err());
    println!("{:?}", actual);
    if let xmlity_quick_xml::Error::NoPossibleVariant { ident } = actual.unwrap_err() {
        assert_eq!(ident, "EnumValue");
    } else {
        panic!("Unexpected error type");
    }
}
