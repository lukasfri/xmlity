use pretty_assertions::assert_eq;

mod common;
use common::{clean_string, quick_xml_deserialize_test, quick_xml_serialize_test};

use xmlity::{
    types::value::{XmlAttribute, XmlChild, XmlElement, XmlText, XmlValue},
    ExpandedName, LocalName, XmlNamespace,
};

const SIMPLE_1D_STRUCT_TEST_XML: &str = r###"
  <to>Tove</to>
"###;

fn xml_value() -> XmlValue {
    XmlValue::Element(
        XmlElement::new(ExpandedName::new(LocalName::new("to").unwrap(), None))
            .with_children(vec![XmlChild::Text(XmlText::new("Tove"))]),
    )
}
#[test]
fn simple_1d_struct_serialize() {
    let actual = quick_xml_serialize_test(xml_value()).unwrap();

    assert_eq!(actual, clean_string(SIMPLE_1D_STRUCT_TEST_XML));
}

#[test]
fn simple_1d_struct_deserialize() {
    let input = clean_string(SIMPLE_1D_STRUCT_TEST_XML);
    let actual: XmlValue = quick_xml_deserialize_test(input.as_str()).unwrap();

    let expected = xml_value();

    assert_eq!(actual, expected);
}

const COMPLEX_XML_EXAMPLE_DESERIALIZE: &str = r###"
<note>
  <to>Tove</to>
  <from>Jani</from>
  <heading>Reminder</heading>
  <body attribute="value">Don't forget me this weekend!</body>
  <testns:test xmlns:testns="http://testns.com">
    Test
  </testns:test>
</note>
"###;

const COMPLEX_XML_EXAMPLE_SERIALIZE: &str = r###"
<note>
    <to>Tove</to>
    <from>Jani</from>
    <heading>Reminder</heading>
    <body attribute="value">Don&apos;t forget me this weekend!</body>
    <a0:test xmlns:a0="http://testns.com">
        Test
    </a0:test>
</note>
"###;

fn complex_xml_value() -> XmlValue {
    XmlValue::Element(
        XmlElement::new(ExpandedName::new(LocalName::new("note").unwrap(), None)).with_children(
            vec![
                XmlChild::Element(
                    XmlElement::new(ExpandedName::new(LocalName::new("to").unwrap(), None))
                        .with_children(vec![XmlChild::Text(XmlText::new("Tove"))]),
                ),
                XmlChild::Element(
                    XmlElement::new(ExpandedName::new(LocalName::new("from").unwrap(), None))
                        .with_children(vec![XmlChild::Text(XmlText::new("Jani"))]),
                ),
                XmlChild::Element(
                    XmlElement::new(ExpandedName::new(LocalName::new("heading").unwrap(), None))
                        .with_children(vec![XmlChild::Text(XmlText::new("Reminder"))]),
                ),
                XmlChild::Element(
                    XmlElement::new(ExpandedName::new(LocalName::new("body").unwrap(), None))
                        .with_attributes(vec![XmlAttribute::new(
                            ExpandedName::new(LocalName::new("attribute").unwrap(), None),
                            "value".to_string(),
                        )])
                        .with_child(XmlText::new("Don't forget me this weekend!")),
                ),
                XmlChild::Element(
                    XmlElement::new(ExpandedName::new(
                        LocalName::new("test").unwrap(),
                        Some(XmlNamespace::new("http://testns.com").unwrap()),
                    ))
                    .with_children(vec![XmlChild::Text(XmlText::new("Test"))]),
                ),
            ],
        ),
    )
}

#[test]
fn complex_xml_value_serialize() {
    let actual = quick_xml_serialize_test(complex_xml_value()).unwrap();

    assert_eq!(actual, clean_string(COMPLEX_XML_EXAMPLE_SERIALIZE));
}

#[test]
fn complex_xml_value_deserialize() {
    let input = clean_string(COMPLEX_XML_EXAMPLE_DESERIALIZE);
    let actual: XmlValue = quick_xml_deserialize_test(input.as_str()).unwrap();

    let expected = complex_xml_value();

    assert_eq!(actual, expected);
}
