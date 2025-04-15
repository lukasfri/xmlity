use pretty_assertions::assert_eq;

mod common;
use common::{clean_string, quick_xml_deserialize_test, quick_xml_serialize_test};

use xmlity::{Deserialize, Serialize};

const EXTENDABLE_STRUCT_TEST_XML: &str = r###"Asdreboot<![CDATA[More]]>Text"###;

const EXTENDABLE_STRUCT_TEST_XML_SER: &str = r###"AsdrebootMoreText"###;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct To(#[xvalue(extendable = true)] String);

fn extendable_struct() -> To {
    To("AsdrebootMoreText".to_string())
}

#[test]
fn extendable_struct_serialize() {
    let actual = quick_xml_serialize_test(extendable_struct()).unwrap();

    assert_eq!(actual, clean_string(EXTENDABLE_STRUCT_TEST_XML_SER));
}

#[test]
fn extendable_struct_deserialize() {
    {
        let mut reader = quick_xml::NsReader::from_reader(EXTENDABLE_STRUCT_TEST_XML.as_bytes());
        while let Ok(event) = reader.read_event() {
            if matches!(event, quick_xml::events::Event::Eof) {
                break;
            }
            println!("{:?}", event);
        }
    }
    let actual: To =
        quick_xml_deserialize_test(clean_string(EXTENDABLE_STRUCT_TEST_XML).as_str()).unwrap();

    let expected = extendable_struct();

    assert_eq!(actual, expected);
}
