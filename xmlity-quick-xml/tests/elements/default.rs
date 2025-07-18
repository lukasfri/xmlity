use crate::define_test;

use xmlity::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[xelement(name = "b")]
pub struct B(#[xvalue(default)] String);

define_test!(
    element_with_text_default_option,
    [
        (B("A".to_string()), "<b>A</b>"),
        (B("".to_string()), "<b/>"),
        (B("".to_string()), "<b/>", "<b></b>")
    ]
);

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[xelement(name = "c")]
pub struct C {
    #[xvalue(default)]
    pub b: B,
}

define_test!(
    element_with_single_child,
    [
        (
            C {
                b: B("A".to_string())
            },
            "<c><b>A</b></c>"
        ),
        (
            C {
                b: B("".to_string())
            },
            "<c><b/></c>",
            "<c/>"
        ),
        (
            C {
                b: B("".to_string())
            },
            "<c><b/></c>"
        )
    ]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "d")]
pub struct D {
    pub b: B,
    #[xvalue(default)]
    pub c: C,
}

define_test!(
    element_with_multiple_children,
    [(
        D {
            b: B("A".to_string()),
            c: C {
                b: B("B".to_string())
            }
        },
        "<d><b>A</b><c><b>B</b></c></d>"
    )]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "e")]
pub struct E {
    #[xvalue(default)]
    pub d: Vec<D>,
}

define_test!(
    element_with_vector_of_children,
    [
        (
            E {
                d: vec![
                    D {
                        b: B("A".to_string()),
                        c: C {
                            b: B("B".to_string())
                        }
                    },
                    D {
                        b: B("C".to_string()),
                        c: C {
                            b: B("D".to_string())
                        }
                    }
                ]
            },
            r#"<e><d><b>A</b><c><b>B</b></c></d><d><b>C</b><c><b>D</b></c></d></e>"#
        ),
        (
            E {
                d: vec![
                    D {
                        b: B("A".to_string()),
                        c: C {
                            b: B("B".to_string())
                        }
                    },
                    D {
                        b: B("C".to_string()),
                        c: C {
                            b: B("D".to_string())
                        }
                    }
                ]
            },
            r#"<e><d><b>A</b><c><b>B</b></c></d><d><b>C</b><c><b>D</b></c></d></e>"#
        ),
        (E { d: vec![] }, "<e/>")
    ]
);
