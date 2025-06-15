use xmlity::{Deserialize, Serialize};

use crate::define_test;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "A")]
pub struct A;
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "B")]
pub struct B;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(order = "strict")]
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
#[xvalue(order = "strict")]
pub struct G {
    #[xelement(name = "E", optional, default)]
    pub e: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(order = "strict")]
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "J")]
pub struct J;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct K {
    #[xvalue(default)]
    pub j: Option<J>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct L {
    pub k: K,
}

define_test!(simple_recursive_child_empty_element, [(K { j: None }, "")]);

define_test!(
    simple_recursive_empty_element,
    [(L { k: K { j: None } }, "")]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct M {
    pub k: Vec<K>,
}

define_test!(
    simple_recursive_empty_element_vec,
    [
        (M { k: vec![] }, ""),
        (
            M {
                k: vec![K { j: Some(J) }]
            },
            "<J/>"
        )
    ]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct N {
    pub j: J,
    pub k: K,
}

define_test!(
    partially_empty_value,
    [
        (
            N {
                j: J,
                k: K { j: None }
            },
            "<J/>"
        ),
        (
            N {
                j: J,
                k: K { j: Some(J) }
            },
            "<J/><J/>"
        )
    ]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "O")]
pub struct O {
    pub j: J,
    pub k: K,
}

define_test!(
    partially_empty_element,
    [
        (
            O {
                j: J,
                k: K { j: None }
            },
            "<O><J/></O>"
        ),
        (
            O {
                j: J,
                k: K { j: Some(J) }
            },
            "<O><J/><J/></O>"
        )
    ]
);
