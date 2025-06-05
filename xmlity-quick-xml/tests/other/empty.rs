use xmlity::{Deserialize, Serialize};

use crate::define_test;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "A")]
pub struct A;
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "B")]
pub struct B;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(order = "loose")]
pub struct F {
    #[xvalue(default)]
    pub child_0: Vec<A>,
    #[xvalue(default)]
    pub any_attribute: Option<Box<B>>,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "C")]
pub struct C;
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "D")]
pub struct D;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(order = "loose")]
pub struct G {
    #[xelement(name = "E", optional, default)]
    pub assert: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(order = "loose")]
pub struct I {
    #[xvalue(default)]
    pub open_content: Option<Box<C>>,
    #[xvalue(default)]
    pub type_def_particle: Option<Box<D>>,
    pub attr_decls: Box<F>,
    pub assertions: Box<G>,
}

define_test!(
    empty_element_1,
    [(
        F {
            child_0: Vec::new(),
            any_attribute: None,
        },
        ""
    )]
);

define_test!(empty_element_2, [(G { assert: None }, "")]);

define_test!(
    recursive_empty_element,
    [(
        I {
            open_content: None,
            type_def_particle: None,
            attr_decls: Box::new(F {
                child_0: Vec::new(),
                any_attribute: None,
            }),
            assertions: Box::new(G { assert: None }),
        },
        ""
    )]
);
