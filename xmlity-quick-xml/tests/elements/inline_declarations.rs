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
    [(
        D {
            b: "A".to_string(),
            c: C { c: "B".to_string() }
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
