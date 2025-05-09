use crate::{define_test, utils::clean_string};

use xmlity::{
    value::{XmlAttribute, XmlChild, XmlDecl, XmlElement, XmlText, XmlValue},
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

define_test!(
    xml_value_1d_element,
    [(xml_value(), clean_string(SIMPLE_1D_STRUCT_TEST_XML))]
);

const COMPLEX_XML_EXAMPLE: &str = r###"
<note>
  <to>Tove</to>
  <from>Jani</from>
  <heading>Reminder</heading>
  <body attribute="value">Don't forget me this weekend!</body>
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

define_test!(
    complex_xml_value,
    [(complex_xml_value(), clean_string(COMPLEX_XML_EXAMPLE))]
);

fn decl_xml_value() -> XmlDecl {
    XmlDecl::new("1.0", Some("UTF-8"), None)
}

define_test!(
    complex_xml_decl,
    [(
        decl_xml_value(),
        r#"<?xml version="1.0" encoding="UTF-8"?>"#
    )]
);
