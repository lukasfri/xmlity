use xmlity::{Deserialize, Serialize};

use crate::define_test;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "any", allow_unknown_children = "any")]
pub struct Any {
    #[xelement(name = "b")]
    b: String,
    #[xelement(name = "d", optional)]
    d: Option<String>,
    e: String,
}

define_test!(
    allow_unknown_children_any,
    [
        (
            Any {
                b: "BVal".to_string(),
                d: None,
                e: "Text".to_string(),
            },
            r###"<any><b>BVal</b>Text</any>"###
        ),
        (
            Any {
                b: "BVal".to_string(),
                d: Some("DVal".to_string()),
                e: "Abc".to_string(),
            },
            r###"<any><b>BVal</b><d>DVal</d>Abc</any>"###,
            r###"<any><b>BVal</b><c/><d>DVal</d>Abc</any>"###
        )
    ]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "at_end", allow_unknown_children = "at_end")]
pub struct AtEnd {
    #[xelement(name = "b")]
    b: String,
    #[xelement(name = "d", optional)]
    d: Option<String>,
    e: String,
}

define_test!(
    allow_unknown_children_at_end,
    [
        (
            AtEnd {
                b: "BVal".to_string(),
                d: None,
                e: "Text".to_string(),
            },
            r###"<at_end><b>BVal</b>Text</at_end>"###
        ),
        (
            AtEnd {
                b: "BVal".to_string(),
                d: Some("DVal".to_string()),
                e: "Abc".to_string(),
            },
            r###"<at_end><b>BVal</b><d>DVal</d>Abc</at_end>"###,
            r###"<at_end><b>BVal</b><d>DVal</d>Abc<c/></at_end>"###
        )
    ]
);

#[test]
fn error_child_in_middle() {
    let xml = r###"<at_end><b>BVal</b><c/><d>DVal</d>Abc</at_end>"###;
    let result: Result<AtEnd, _> = xmlity_quick_xml::de::from_str(xml);
    assert!(result.is_err());
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "none", allow_unknown_children = "none")]
pub struct NoUnknown {
    #[xelement(name = "b")]
    b: String,
    #[xelement(name = "d", optional)]
    d: Option<String>,
    e: String,
}

define_test!(
    allow_unknown_children_none,
    [
        (
            NoUnknown {
                b: "BVal".to_string(),
                d: None,
                e: "Text".to_string(),
            },
            r###"<none><b>BVal</b>Text</none>"###
        ),
        (
            NoUnknown {
                b: "BVal".to_string(),
                d: Some("DVal".to_string()),
                e: "Abc".to_string(),
            },
            r###"<none><b>BVal</b><d>DVal</d>Abc</none>"###
        )
    ]
);

#[test]
fn error_child_in_middle_none() {
    let xml = r###"<none><b>BVal</b><c/><d>DVal</d>Abc</none>"###;
    let result: Result<NoUnknown, _> = xmlity_quick_xml::de::from_str(xml);
    assert!(result.is_err());
}

#[test]
fn error_child_in_end() {
    let xml = r###"<none><b>BVal</b><d>DVal</d>Abc<c/></none>"###;
    let result: Result<NoUnknown, _> = xmlity_quick_xml::de::from_str(xml);
    assert!(result.is_err());
}
