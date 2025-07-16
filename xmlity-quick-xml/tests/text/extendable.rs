use crate::define_test;

use xmlity::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ExtendableText(#[xvalue(extendable = true)] String);

define_test!(
    extendable_text,
    [
        (
            ExtendableText("BeforeInsideAfter".to_string()),
            "BeforeInsideAfter",
            "Before<![CDATA[Inside]]>After"
        ),
        (ExtendableText("Text".to_string()), "Text")
    ]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ExtendableVec(#[xvalue(extendable = true)] Vec<String>);

define_test!(
    extendable_vec,
    [
        (
            ExtendableVec(vec![
                "Before".to_string(),
                "Inside".to_string(),
                "After".to_string()
            ]),
            "BeforeInsideAfter",
            "Before<![CDATA[Inside]]>After"
        ),
        (ExtendableVec(vec!["Text".to_string()]), "Text")
    ]
);
