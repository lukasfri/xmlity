use xmlity::{Deserialize, Serialize};

use crate::define_test;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "a")]
pub struct A;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "b")]
pub struct B;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "C")]
pub struct C {
    #[xvalue(default)]
    pub a: Option<A>,
    #[xvalue(default)]
    pub b: Option<B>,
}

define_test!(
    option_element,
    [
        (
            C {
                a: Some(A),
                b: None
            },
            "<C><a/></C>"
        ),
        (
            C {
                a: None,
                b: Some(B)
            },
            "<C><b/></C>"
        )
    ]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct D {
    #[xvalue(default)]
    pub a: Option<A>,
    #[xvalue(default)]
    pub b: Option<B>,
}

define_test!(
    option_value,
    [
        (
            D {
                a: Some(A),
                b: None
            },
            "<a/>"
        ),
        (
            D {
                a: None,
                b: Some(B)
            },
            "<b/>"
        )
    ]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "e")]
pub struct E {
    pub text: Option<String>,
}

define_test!(
    option_text,
    [
        (
            E {
                text: Some("Text".to_string())
            },
            "<e>Text</e>"
        ),
        (E { text: None }, "<e/>"),
        (E { text: None }, "<e/>", "<e></e>")
    ]
);
