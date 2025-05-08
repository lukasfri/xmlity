use crate::define_test;

use xmlity::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "c")]
pub struct C {
    #[xattribute(name = "b")]
    pub c: String,
}

define_test!(
    element_with_single_child,
    [(C { c: "A".to_string() }, r#"<c b="A"/>"#)]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "d")]
pub struct D {
    #[xattribute(name = "b")]
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
        r#"<d b="A"><c b="B"/></d>"#
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
        r#"<e><d b="A"><c b="B"/></d><d b="C"><c b="D"/></d></e>"#
    )]
);
