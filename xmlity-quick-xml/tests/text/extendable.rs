use crate::define_test;

use xmlity::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ExtendableText(#[xvalue(extendable = true)] String);

define_test!(
    extendable_struct,
    [
        (
            ExtendableText("BeforeInsideAfter".to_string()),
            "BeforeInsideAfter",
            "Before<![CDATA[Inside]]>After"
        ),
        (ExtendableText("Text".to_string()), "Text")
    ]
);
