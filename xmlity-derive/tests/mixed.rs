//! Tests for content that is mixed, such as text and elements in the same children.
use pretty_assertions::assert_eq;

mod common;
use common::{clean_string, quick_xml_deserialize_test, quick_xml_serialize_test};

use xmlity::types::utils::CData;
use xmlity_derive::{Deserialize, Serialize};

const SIMPLE_MIXED_STRUCT_TEST_XML: &str = r###"
<mixed>
  Text
  <elem>Content</elem>
  Text2
</mixed> 
"###;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "elem")]
pub struct Elem(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "mixed")]
pub struct Mixed {
    pub text1: String,
    pub elem: Elem,
    pub text2: String,
}

fn simple_mixed_struct() -> Mixed {
    Mixed {
        text1: "Text".to_string(),
        elem: Elem("Content".to_string()),
        text2: "Text2".to_string(),
    }
}

#[test]
fn simple_mixed_struct_serialize() {
    let actual = quick_xml_serialize_test(simple_mixed_struct()).expect("Failed to serialize");

    let expected = SIMPLE_MIXED_STRUCT_TEST_XML
        .lines()
        .map(str::trim)
        .collect::<String>();

    assert_eq!(actual, expected);
}

#[test]
fn simple_mixed_struct_deserialize() {
    let actual: Mixed = quick_xml_deserialize_test(
        SIMPLE_MIXED_STRUCT_TEST_XML
            .lines()
            .map(str::trim)
            .collect::<String>()
            .as_str(),
    )
    .expect("Failed to deserialize");

    let expected = simple_mixed_struct();

    assert_eq!(actual, expected);
}

const MIXED_CDATA_STRUCT_TEST_XML: &str = r###"
<mixed>
Text<![CDATA[More]]>
<elem>Content</elem>
Text2
</mixed>
"###;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "mixed")]
pub struct MixedCData {
    pub text1: String,
    pub cdata: CData<String>,
    pub elem: Elem,
    pub text2: String,
}

fn mixed_cdata_struct() -> MixedCData {
    MixedCData {
        text1: "Text".to_string(),
        cdata: CData("More".to_string()),
        elem: Elem("Content".to_string()),
        text2: "Text2".to_string(),
    }
}

#[test]
fn mixed_cdata_struct_serialize() {
    let actual = quick_xml_serialize_test(mixed_cdata_struct()).expect("Failed to serialize");

    let expected = MIXED_CDATA_STRUCT_TEST_XML
        .lines()
        .map(str::trim)
        .collect::<String>();

    assert_eq!(actual, expected);
}

#[test]
fn mixed_cdata_struct_deserialize() {
    let actual: MixedCData = quick_xml_deserialize_test(
        MIXED_CDATA_STRUCT_TEST_XML
            .lines()
            .map(str::trim)
            .collect::<String>()
            .as_str(),
    )
    .expect("Failed to deserialize");

    let expected = mixed_cdata_struct();

    assert_eq!(actual, expected);
}

const MIXED_CDATA_CONCAT_STRUCT_TEST_XML: &str = r###"
<mixed>
Text1<![CDATA[Text2]]>
<elem>Text3</elem>
Text4
</mixed>
"###;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "mixed")]
pub struct MixedCDataConcat {
    pub text1: String,
    pub text2: CData<String>,
    pub elem: Elem,
    pub text4: String,
}

fn mixed_cdata_concat_struct() -> MixedCDataConcat {
    MixedCDataConcat {
        text1: "Text1".to_string(),
        text2: CData("Text2".to_string()),
        elem: Elem("Text3".to_string()),
        text4: "Text4".to_string(),
    }
}

#[test]
fn mixed_cdata_concat_struct_serialize() {
    let actual =
        quick_xml_serialize_test(mixed_cdata_concat_struct()).expect("Failed to serialize");

    let expected = MIXED_CDATA_CONCAT_STRUCT_TEST_XML
        .lines()
        .map(str::trim)
        .collect::<String>();

    assert_eq!(actual, expected);
}

#[test]
fn mixed_cdata_concat_struct_deserialize() {
    let actual: MixedCDataConcat = quick_xml_deserialize_test(
        MIXED_CDATA_CONCAT_STRUCT_TEST_XML
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<String>()
            .as_str(),
    )
    .expect("Failed to deserialize");

    assert_eq!(actual, mixed_cdata_concat_struct());
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Variant {
    CData(CData<String>),
    String(String),
}

#[test]
fn variant_enum_serialize() {
    let actual = quick_xml_serialize_test(Variant::String("Text1".to_string()))
        .expect("Failed to serialize");

    let expected = clean_string("Text1");

    assert_eq!(actual, expected.as_str());
}

#[test]
fn variant_enum_deserialize() {
    let actual = quick_xml_deserialize_test::<Variant>("Text1").expect("Failed to deserialize");

    let expected = Variant::String("Text1".to_string());

    assert_eq!(actual, expected);
}

const MIXED_CDATA_LIST_STRUCT_TEST_XML: &str = r###"
<list>
Text1<![CDATA[Text2]]>
</list>
"###;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "list")]
pub struct VariantList {
    pub text1: Vec<Variant>,
}

fn mixed_cdata_list_struct() -> VariantList {
    VariantList {
        text1: vec![
            Variant::String("Text1".to_string()),
            Variant::CData(CData("Text2".to_string())),
        ],
    }
}

#[test]
fn mixed_cdata_list_struct_serialize() {
    let actual = quick_xml_serialize_test(mixed_cdata_list_struct()).expect("Failed to serialize");

    let expected = clean_string(MIXED_CDATA_LIST_STRUCT_TEST_XML);

    assert_eq!(actual, expected.as_str());
}

#[test]
fn mixed_cdata_list_struct_deserialize() {
    let actual = quick_xml_deserialize_test::<VariantList>(
        clean_string(MIXED_CDATA_LIST_STRUCT_TEST_XML).as_str(),
    )
    .expect("Failed to deserialize");

    let expected = mixed_cdata_list_struct();

    assert_eq!(actual, expected);
}

const MIXED_CDATA_SEPARATED_STRUCT_TEST_XML: &str = r###"
<mixed>
Text1<![CDATA[Text2]]>
<elem>Text3</elem>
Text4
</mixed>
"###;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "mixed")]
pub struct MixedCDataSeparated {
    pub text1: Vec<Variant>,
    pub elem: Elem,
    pub text2: String,
}

fn mixed_cdata_separated_struct() -> MixedCDataSeparated {
    MixedCDataSeparated {
        text1: vec![
            Variant::String("Text1".to_string()),
            Variant::CData(CData("Text2".to_string())),
        ],
        elem: Elem("Text3".to_string()),
        text2: "Text4".to_string(),
    }
}

#[test]
fn mixed_cdata_separated_struct_serialize() {
    let actual =
        quick_xml_serialize_test(mixed_cdata_separated_struct()).expect("Failed to serialize");

    let expected = clean_string(MIXED_CDATA_SEPARATED_STRUCT_TEST_XML);

    assert_eq!(actual, expected.as_str());
}

#[test]
fn mixed_cdata_separated_struct_deserialize() {
    let actual = quick_xml_deserialize_test::<MixedCDataSeparated>(
        clean_string(MIXED_CDATA_SEPARATED_STRUCT_TEST_XML).as_str(),
    )
    .expect("Failed to deserialize");

    let expected = mixed_cdata_separated_struct();

    assert_eq!(actual, expected);
}
