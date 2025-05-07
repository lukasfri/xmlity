use rstest::rstest;
use std::str::FromStr;
use xmlity::{
    types::value::{
        XmlAttribute, XmlCData, XmlChild, XmlComment, XmlElement, XmlProcessingInstruction, XmlSeq,
        XmlText, XmlValue,
    },
    xml, XmlNamespace,
};
use xmlity::{ExpandedName, LocalName};

#[rstest]
#[case::comment(xml!(<!--"Test"-->), XmlComment::new(b"Test"))]
#[case::pi(xml!(<?"Test" "Content"?>), XmlProcessingInstruction::new(b"Test", b"Content"))]
#[case::basic_element(xml!(<"Test"/>), XmlElement::new(ExpandedName::new(LocalName::from_str("Test").unwrap(), None)))]
#[case::element_with_child(xml!(<"Test">["Childtext"]</"Test">), XmlElement::new(ExpandedName::new(LocalName::from_str("Test").unwrap(), None))
    .with_child(XmlChild::from(XmlText::new("Childtext"))))]
#[case::two_comments(xml!(<!--"Test"--><!--"Test"-->), XmlSeq::from_iter([
    XmlValue::from(XmlComment::new(b"Test")),
    XmlValue::from(XmlComment::new(b"Test"))
]))]
#[case::cdata(xml!(<![CDATA["some stuff"]]>), XmlCData(b"some stuff".to_vec()))]
#[case::note_element(xml!(<"note">[
    <"to">["Tove"]</"to">
    <"from">["Jani"]</"from">
    <"heading">["Reminder"]</"heading">
    <"body">["Don't forget me this weekend!"]</"body">
]</"note">
), XmlElement::new(ExpandedName::new(LocalName::new("note").unwrap(), None))
    .with_child(XmlElement::new(ExpandedName::new(LocalName::new("to").unwrap(), None))
        .with_child(XmlText::new("Tove")))
    .with_child(XmlElement::new(ExpandedName::new(LocalName::new("from").unwrap(), None))
        .with_child(XmlText::new("Jani")))
    .with_child(XmlElement::new(ExpandedName::new(LocalName::new("heading").unwrap(), None))
        .with_child(XmlText::new("Reminder")))
    .with_child(XmlElement::new(ExpandedName::new(LocalName::new("body").unwrap(), None))
        .with_child(XmlText::new("Don't forget me this weekend!")))
)]
#[case::note_element_with_namespace(xml!(<"Test":"http://example.com">["Childtext"]</"Test">), XmlElement::new(ExpandedName::new(LocalName::from_str("Test").unwrap(), Some(XmlNamespace::from_str("http://example.com").unwrap())))
    .with_child(XmlText::new("Childtext"))
)]
#[case::note_element_with_namespace_and_attribute(xml!(<"Test":"http://example.com" "abc"="def">["Childtext"]</"Test">), XmlElement::new(ExpandedName::new(LocalName::from_str("Test").unwrap(), Some(XmlNamespace::from_str("http://example.com").unwrap())))
.with_attribute(XmlAttribute::new(
     ExpandedName::new(LocalName::from_str("abc").unwrap(), Some(XmlNamespace::from_str("http://example.com").unwrap())),
     "def".to_string()
))
.with_child(XmlText::new("Childtext")))]
#[case::note_element_with_namespace_and_attributes(xml!(<"Test":"http://example.com" "abc"="def" "ghi"="jkl">["Childtext"]</"Test">), XmlElement::new( ExpandedName::new(LocalName::from_str("Test").unwrap(), Some(XmlNamespace::from_str("http://example.com").unwrap())))
    .with_attribute(XmlAttribute::new(
        ExpandedName::new(LocalName::from_str("abc").unwrap(), Some(XmlNamespace::from_str("http://example.com").unwrap())),
        "def"
    ))
    .with_attribute(XmlAttribute::new(
        ExpandedName::new(LocalName::from_str("ghi").unwrap(), Some(XmlNamespace::from_str("http://example.com").unwrap())),
        "jkl"
    ))
    .with_child(XmlText::new("Childtext")))]
#[case::note_element_with_namespace_and_namespaced_attribute(xml!(<"Test":"http://example.com" "abc":"http://example.org/this/is/a/namespace"="def">["Childtext"]</"Test">), XmlElement::new(ExpandedName::new(LocalName::from_str("Test").unwrap(), Some(XmlNamespace::from_str("http://example.com").unwrap())))
.with_attribute(XmlAttribute::new(
    ExpandedName::new(LocalName::from_str("abc").unwrap(), Some(XmlNamespace::from_str("http://example.org/this/is/a/namespace").unwrap())),
    "def"
))
.with_child(XmlText::new("Childtext"))
)]
fn xml_macro_equals<T: PartialEq + std::fmt::Debug>(#[case] t1: T, #[case] t2: T) {
    assert_eq!(t1, t2);
}

#[rstest]
#[case::xml_element_name_end_mismatch(xml!(<"Test">["Childtext"]</"Test2">))]
#[should_panic]
fn xml_macro_panics<T: PartialEq + std::fmt::Debug>(#[case] _t1: T) {}
