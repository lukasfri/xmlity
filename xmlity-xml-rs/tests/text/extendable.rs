use crate::define_test;

use xmlity::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ExtendableText(#[xvalue(extendable = true)] String);

fn extendable_struct() -> ExtendableText {
    ExtendableText("AsdrebootMoreText".to_string())
}

define_test!(
    extendable_struct,
    [
        (
            extendable_struct(),
            "AsdrebootMoreText",
            "Asdreboot<![CDATA[More]]>Text"
        ),
        (extendable_struct(), "AsdrebootMoreText")
    ]
);
