use crate::define_test;

use xmlity::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "b")]
pub struct B(String);

define_test!(element_with_text, [(B("A".to_string()), "<b>A</b>")]);

#[rstest::rstest]
#[case("<b></b>")]
#[case("<b><c></c></b>")]
fn wrong_deserialize(#[case] xml: &str) {
    let actual: Result<B, _> = crate::utils::quick_xml_deserialize_test(xml);

    assert!(actual.is_err());
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "c")]
pub struct C {
    pub c: B,
}

define_test!(
    element_with_single_child,
    [(
        C {
            c: B("A".to_string())
        },
        "<c><b>A</b></c>"
    )]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "d")]
pub struct D {
    pub b: B,
    pub c: C,
}

define_test!(
    element_with_multiple_children,
    [(
        D {
            b: B("A".to_string()),
            c: C {
                c: B("B".to_string())
            }
        },
        "<d><b>A</b><c><b>B</b></c></d>"
    )]
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
                    b: B("A".to_string()),
                    c: C {
                        c: B("B".to_string())
                    }
                },
                D {
                    b: B("C".to_string()),
                    c: C {
                        c: B("D".to_string())
                    }
                }
            ]
        },
        r#"<e><d><b>A</b><c><b>B</b></c></d><d><b>C</b><c><b>D</b></c></d></e>"#
    )]
);
