use xmlity::{DeserializationGroup, Deserialize, SerializationGroup, Serialize};

use crate::define_test;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(order = "strict")]
pub struct CombinedValue {
    #[xvalue(default)]
    pub a_list: Vec<A>,
    pub b_parent: BParent,
}

#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
#[xgroup(children_order = "strict")]
pub struct CombinedValueGroup {
    pub child_1: CombinedValue,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(order = "strict")]
pub struct BParent {
    #[xelement(name = "b", optional, default)]
    pub b: Option<()>,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "a")]
pub struct A;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "c")]
pub struct C(#[xgroup] pub CombinedValueGroup);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "d")]
pub struct D(pub CombinedValue);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "e", children_order = "strict")]
pub struct E {
    #[xvalue(default)]
    pub a_list: Vec<A>,
    pub b_parent: BParent,
}

define_test!(
    combined_value,
    [(
        CombinedValue {
            a_list: vec![A, A],
            b_parent: BParent { b: None },
        },
        r###"<a/><a/>"###
    )]
);

define_test!(
    c,
    [(
        C(CombinedValueGroup {
            child_1: CombinedValue {
                a_list: vec![A, A],
                b_parent: BParent { b: None },
            }
        }),
        r###"<c><a/><a/></c>"###
    )]
);

define_test!(
    d,
    [(
        D(CombinedValue {
            a_list: vec![A, A],
            b_parent: BParent { b: None },
        }),
        r###"<d><a/><a/></d>"###
    )]
);

define_test!(
    e,
    [(
        E {
            a_list: vec![A, A],
            b_parent: BParent { b: None },
        },
        r###"<e><a/><a/></e>"###
    )]
);
