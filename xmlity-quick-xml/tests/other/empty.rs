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
    pub a: Vec<A>,
    #[xvalue(default)]
    pub b: Option<Box<B>>,
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
    pub e: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(order = "loose")]
pub struct I {
    #[xvalue(default)]
    pub c: Option<Box<C>>,
    #[xvalue(default)]
    pub d: Option<Box<D>>,
    pub f: Box<F>,
    pub g: Box<G>,
}

define_test!(
    empty_element_1,
    [(
        F {
            a: Vec::new(),
            b: None,
        },
        ""
    )]
);

define_test!(empty_element_2, [(G { e: None }, "")]);

define_test!(
    recursive_empty_element,
    [(
        I {
            c: None,
            d: None,
            f: Box::new(F {
                a: Vec::new(),
                b: None,
            }),
            g: Box::new(G { e: None }),
        },
        ""
    )]
);
