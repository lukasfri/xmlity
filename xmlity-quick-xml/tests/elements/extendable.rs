use crate::define_test;

use xmlity::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "a")]
pub struct A(#[xvalue(extendable = true)] String);

impl Extend<Self> for A {
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(|a| a.0));
    }
}

fn extendable_struct() -> ExtendableA {
    ExtendableA(A("AsdrebootMoreText".to_string()))
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ExtendableA(#[xvalue(extendable = true)] A);

define_test!(
    extendable_struct,
    [
        (
            extendable_struct(),
            "<a>AsdrebootMoreText</a>",
            "<a>Asdreboot<![CDATA[More]]></a><a>Text</a>"
        ),
        (extendable_struct(), "<a>AsdrebootMoreText</a>"),
        (
            extendable_struct(),
            "<a>AsdrebootMoreText</a>",
            "<a>Asd</a><a>reboot</a><a>More</a><a>Text</a>"
        )
    ]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "b")]
pub struct B(#[xvalue(extendable = "iterator")] Vec<String>);

fn extendable_vec1() -> B {
    B(vec![
        "Asdreboot".to_string(),
        "More".to_string(),
        "Text".to_string(),
    ])
}

fn extendable_vec2() -> B {
    B(vec!["Asd".to_string()])
}

define_test!(
    extendable_vec,
    [
        (
            extendable_vec1(),
            "<b>AsdrebootMoreText</b>",
            "<b>Asdreboot<![CDATA[More]]>Text</b>"
        ),
        (extendable_vec2(), "<b>Asd</b>")
    ]
);
