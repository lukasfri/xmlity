use pretty_assertions::assert_eq;

mod common;
use common::{quick_xml_deserialize_test, quick_xml_serialize_test};

const TEXT_XML: &str = "Alpha";

#[test]
fn simple_1d_struct_serialize() {
    let actual = quick_xml_serialize_test("Alpha").unwrap();

    assert_eq!(actual, TEXT_XML);
}

#[test]
fn simple_1d_struct_deserialize() {
    let actual: String = quick_xml_deserialize_test(TEXT_XML).unwrap();

    let expected = "Alpha";

    assert_eq!(actual, expected);
}
