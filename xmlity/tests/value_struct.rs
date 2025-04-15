//! Tests for basic functionality. These tests are the most basic and do not include any attributes. They are simply used to test the default behavior of the library.
use pretty_assertions::assert_eq;

use xmlity::{
    types::{utils::CData, value::XmlSeq},
    Deserialize, Serialize, XmlValue,
};

fn simple_value_struct_xml() -> XmlSeq<XmlValue> {
    xmlity::xml!("Test1"<![CDATA["Test2"]]>"Test3")
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Test(String, CData<String>, String);

fn simple_value_struct() -> Test {
    Test(
        "Test1".to_owned(),
        CData("Test2".to_owned()),
        "Test3".to_owned(),
    )
}

#[test]
fn simple_value_struct_serialize() {
    let mut actual = XmlSeq::default();
    simple_value_struct().serialize(&mut &mut actual).unwrap();

    assert_eq!(actual, simple_value_struct_xml());
}

#[test]
fn simple_value_struct_deserialize() {
    let source = simple_value_struct_xml();
    let actual = Test::deserialize(&source).unwrap();

    let expected = simple_value_struct();

    assert_eq!(actual, expected);
}
