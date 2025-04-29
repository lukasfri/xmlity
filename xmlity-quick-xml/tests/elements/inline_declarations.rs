use crate::define_test;

use xmlity::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "c")]
pub struct C {
    #[xelement(name = "b")]
    pub c: String,
}

define_test!(
    element_with_single_child,
    [(C { c: "A".to_string() }, "<c><b>A</b></c>")]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "d")]
pub struct D {
    #[xelement(name = "b")]
    pub b: String,
    pub c: C,
}

define_test!(
    element_with_multiple_children,
    [
        (
            D {
                b: "A".to_string(),
                c: C { c: "B".to_string() }
            },
            "<d><b>A</b><c><b>B</b></c></d>"
        ),
        (
            vec![
                D {
                    b: "A".to_string(),
                    c: C { c: "B".to_string() }
                },
                D {
                    b: "C".to_string(),
                    c: C { c: "D".to_string() }
                }
            ],
            "<d><b>A</b><c><b>B</b></c></d><d><b>C</b><c><b>D</b></c></d>"
        )
    ]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "e")]
pub struct E {
    pub d: Vec<D>,
}

define_test!(
    element_with_vector_of_children,
    [(
        E {
            d: vec![
                D {
                    b: "A".to_string(),
                    c: C { c: "B".to_string() }
                },
                D {
                    b: "C".to_string(),
                    c: C { c: "D".to_string() }
                }
            ]
        },
        r#"<e><d><b>A</b><c><b>B</b></c></d><d><b>C</b><c><b>D</b></c></d></e>"#
    )]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct F {
    #[xvalue(extendable = "iterator")]
    pub g: Vec<D>,
}

define_test!(
    element_with_extendable,
    [(
        F {
            g: vec![
                D {
                    b: "A".to_string(),
                    c: C { c: "B".to_string() }
                },
                D {
                    b: "C".to_string(),
                    c: C { c: "D".to_string() }
                }
            ]
        },
        r#"<d><b>A</b><c><b>B</b></c></d><d><b>C</b><c><b>D</b></c></d>"#
    )]
);

#[rstest::rstest]
#[case("")]
#[case(r#"<b>A</b><c><b>B</b></c><b>C</b><c><b>D</b></c>"#)]
fn element_with_extendable_wrong_deserialize(#[case] xml: &str) {
    let actual: Result<D, _> = crate::utils::quick_xml_deserialize_test(xml);

    assert!(actual.is_err());
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct H {
    #[xelement(name = "g")]
    pub g: Vec<D>,
}

define_test!(
    multiple_elements_with_extendable,
    [(
        H {
            g: vec![
                D {
                    b: "A".to_string(),
                    c: C { c: "B".to_string() }
                },
                D {
                    b: "C".to_string(),
                    c: C { c: "D".to_string() }
                }
            ]
        },
        r#"<g><d><b>A</b><c><b>B</b></c></d><d><b>C</b><c><b>D</b></c></d></g>"#
    )]
);
