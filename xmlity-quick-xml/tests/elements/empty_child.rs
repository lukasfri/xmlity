use xmlity::{Deserialize, Serialize};

use crate::define_test;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "a")]
pub struct A {
    text: String,
}

define_test!(
    empty_element,
    [(
        A {
            text: "".to_string(),
        },
        "<a/>"
    )]
);
