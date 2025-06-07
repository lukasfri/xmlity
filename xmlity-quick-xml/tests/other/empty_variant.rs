use xmlity::{Deserialize, Serialize};

use crate::define_test;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "A")]
pub struct A;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct B {
    #[xvalue(default)]
    pub j: Option<A>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum C {
    A(A),
    B(B),
}

define_test!(empty_value_in_enum, [(C::B(B { j: None }), "")]);
