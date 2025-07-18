use xmlity::{Deserialize, Serialize};

use crate::define_test;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "a")]
pub struct A;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "b", ignore_comments = "any")]
pub struct B(pub A);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "c", ignore_comments = "none")]
pub struct C(pub A);

define_test!(
    ignore_any_comments,
    [
        (B(A), "<b><a/></b>", "<b><!-- comment --><a/></b>"),
        (C(A), "<c><a/></c>")
    ]
);

#[test]
fn error_on_whitespace_in_c() {
    let xml = "<c><!-- comment --><a/><!-- comment --></c>";

    let err = xmlity_quick_xml::from_str::<C>(xml).unwrap_err();

    println!("{err}");
}
