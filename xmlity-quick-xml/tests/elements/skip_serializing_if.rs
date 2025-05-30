use crate::define_test;

use xmlity::{Deserialize, Serialize};

fn is_coolio(s: &str) -> bool {
    s == "coolio"
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "c")]
pub struct C {
    #[xvalue(skip_serializing_if = "is_coolio")]
    pub c: String,
}

define_test!(
    skip_serializing_if_string,
    [
        (C { c: "A".to_string() }, "<c>A</c>"),
        (
            C {
                c: "coolio".to_string()
            },
            "<c/>",
            "<c>coolio</c>"
        )
    ]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "e")]
pub struct E {
    #[xelement(name = "d", skip_serializing_if = "is_coolio")]
    pub d: String,
}

define_test!(
    skip_serializing_if_inline_element_string,
    [
        (E { d: "A".to_string() }, "<e><d>A</d></e>"),
        (
            E {
                d: "coolio".to_string()
            },
            "<e/>",
            "<e><d>coolio</d></e>"
        )
    ]
);
