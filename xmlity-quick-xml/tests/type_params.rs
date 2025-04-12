//! Tests for basic functionality. These tests are the most basic and do not include any attributes. They are simply used to test the default behavior of the library.
use pretty_assertions::assert_eq;

mod common;
use common::{clean_string, quick_xml_deserialize_test, quick_xml_serialize_test};

use xmlity::{
    DeserializationGroup, Deserialize, DeserializeOwned, SerializationGroup, Serialize,
    SerializeAttribute,
};

const SIMPLE_2D_STRUCT_TEST_XML: &str = r###"
<note to="Tove">
  <from>Jani</from>
  <heading>Reminder</heading>
  <body>Dont forget me this weekend!</body>
</note>
"###;

#[derive(Debug, PartialEq, SerializeAttribute, Deserialize)]
#[xattribute(name = "to")]
pub struct To(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "from")]
pub struct From(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "heading")]
pub struct Heading(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "body")]
pub struct Body(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "note")]
pub struct Note<T: SerializeAttribute + DeserializeOwned> {
    #[xattribute]
    pub to: T,
    pub from: From,
    pub heading: Heading,
    pub body: Body,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum NoteEnum<T: SerializeAttribute + DeserializeOwned> {
    Note(Note<T>),
}

#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
#[xelement(name = "note")]
pub struct NoteGroup<T: SerializeAttribute + DeserializeOwned> {
    #[xattribute]
    pub to: T,
    pub from: From,
    pub heading: Heading,
    pub body: Body,
}

fn simple_2d_struct_result() -> Note<To> {
    Note {
        to: To("Tove".to_string()),
        from: From("Jani".to_string()),
        heading: Heading("Reminder".to_string()),
        body: Body("Dont forget me this weekend!".to_string()),
    }
}

#[test]
fn simple_2d_struct_serialize() {
    let actual = quick_xml_serialize_test(simple_2d_struct_result()).unwrap();

    let expected = clean_string(SIMPLE_2D_STRUCT_TEST_XML);

    assert_eq!(actual, expected);
}

#[test]
fn simple_2d_struct_deserialize() {
    let actual: Note<To> =
        quick_xml_deserialize_test(clean_string(SIMPLE_2D_STRUCT_TEST_XML).as_str()).unwrap();

    let expected = simple_2d_struct_result();

    assert_eq!(actual, expected);
}
