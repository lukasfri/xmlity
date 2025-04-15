use pretty_assertions::assert_eq;

mod common;
use common::{clean_string, quick_xml_deserialize_test, quick_xml_serialize_test};

use xmlity::{Deserialize, Serialize};

const SIMPLE_1D_STRUCT_TEST_XML: &str = r###"Asdreboot<![CDATA[More]]>Text"###;

const SIMPLE_1D_STRUCT_TEST_XML_SER: &str = r###"AsdrebootMoreText"###;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct To(#[xvalue(extendable = true)] String);

fn simple_1d_struct() -> To {
    To("AsdrebootMoreText".to_string())
}

#[test]
fn simple_1d_struct_serialize() {
    let actual = quick_xml_serialize_test(simple_1d_struct()).unwrap();

    assert_eq!(actual, clean_string(SIMPLE_1D_STRUCT_TEST_XML_SER));
}

#[test]
fn simple_1d_struct_deserialize() {
    {
        let mut reader = quick_xml::NsReader::from_reader(SIMPLE_1D_STRUCT_TEST_XML.as_bytes());
        while let Ok(event) = reader.read_event() {
            if matches!(event, quick_xml::events::Event::Eof) {
                break;
            }
            println!("{:?}", event);
        }
    }
    let actual: To =
        quick_xml_deserialize_test(clean_string(SIMPLE_1D_STRUCT_TEST_XML).as_str()).unwrap();

    let expected = simple_1d_struct();

    assert_eq!(actual, expected);
}
