use crate::define_test;

use xmlity::{Deserialize, Serialize, SerializeAttribute};

#[derive(Debug, PartialEq, SerializeAttribute, Deserialize)]
#[xattribute(name = "b")]
pub struct B(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "C")]
pub struct C {
    #[xattribute(deferred = true)]
    pub b: B,
}

define_test!(
    one_attribute,
    [(
        C {
            b: B("A".to_string())
        },
        r#"<C b="A"/>"#
    )]
);

#[derive(Debug, PartialEq, SerializeAttribute, Deserialize)]
#[xattribute(name = "D")]
pub struct D(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "E")]
pub struct E {
    #[xattribute(deferred = true)]
    pub b: B,
    #[xattribute(deferred = true)]
    pub d: D,
}

define_test!(
    two_attributes,
    [(
        E {
            b: B("A".to_string()),
            d: D("B".to_string())
        },
        r#"<E b="A" D="B"/>"#
    )]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "F")]
pub struct F {
    #[xattribute(deferred = true)]
    pub b: B,
    pub c: Vec<C>,
    pub e: E,
}

define_test!(
    element_with_children_and_attributes,
    [(
        F {
            b: B("A".to_string()),
            c: vec![
                C {
                    b: B("B".to_string())
                },
                C {
                    b: B("B".to_string())
                }
            ],
            e: E {
                b: B("A".to_string()),
                d: D("B".to_string())
            }
        },
        r#"<F b="A"><C b="B"/><C b="B"/><E b="A" D="B"/></F>"#
    )]
);
