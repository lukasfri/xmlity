use crate::define_test;

use xmlity::types::utils::CData;
use xmlity::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "elem")]
pub struct Elem(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "mixed")]
pub struct Mixed {
    pub text1: String,
    pub elem: Elem,
    pub text2: String,
}

fn simple_mixed_struct() -> Mixed {
    Mixed {
        text1: "Text".to_string(),
        elem: Elem("Content".to_string()),
        text2: "Text2".to_string(),
    }
}

define_test!(
    mixed_text_and_element,
    [(
        simple_mixed_struct(),
        "<mixed>Text<elem>Content</elem>Text2</mixed>"
    )]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "mixed")]
pub struct MixedCData {
    pub text1: String,
    pub cdata: CData<String>,
    pub elem: Elem,
    pub text2: String,
}

fn mixed_cdata_struct() -> MixedCData {
    MixedCData {
        text1: "Text".to_string(),
        cdata: CData("More".to_string()),
        elem: Elem("Content".to_string()),
        text2: "Text2".to_string(),
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "mixed")]
pub struct MixedCDataConcat {
    pub text1: String,
    pub text2: CData<String>,
    pub elem: Elem,
    pub text4: String,
}

fn mixed_cdata_concat_struct() -> MixedCDataConcat {
    MixedCDataConcat {
        text1: "Text1".to_string(),
        text2: CData("Text2".to_string()),
        elem: Elem("Text3".to_string()),
        text4: "Text4".to_string(),
    }
}

define_test!(
    mixed_text_cdata_and_element,
    [
        (
            mixed_cdata_struct(),
            "<mixed>Text<![CDATA[More]]><elem>Content</elem>Text2</mixed>"
        ),
        (
            mixed_cdata_concat_struct(),
            "<mixed>Text1<![CDATA[Text2]]><elem>Text3</elem>Text4</mixed>"
        )
    ]
);
