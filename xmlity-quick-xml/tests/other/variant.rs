use crate::define_test;
pub use xmlity::types::utils::CData;
pub use xmlity::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum CDataOrText {
    CData(CData<String>),
    String(String),
}

define_test!(
    cdata_or_text_enum,
    [
        (CDataOrText::String("Text".to_owned()), "Text"),
        (
            CDataOrText::CData(CData("CData".to_owned())),
            "<![CDATA[CData]]>"
        ),
        (CDataOrText::String("".to_owned()), "")
    ]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "list")]
pub struct VariantList {
    pub text1: Vec<CDataOrText>,
}

fn mixed_cdata_list() -> VariantList {
    VariantList {
        text1: vec![
            CDataOrText::String("Text1".to_string()),
            CDataOrText::CData(CData("Text2".to_string())),
        ],
    }
}

define_test!(
    mixed_cdata_list_struct,
    [(mixed_cdata_list(), "<list>Text1<![CDATA[Text2]]></list>")]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "elem")]
pub struct Elem(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "mixed")]
pub struct MixedCDataSeparated {
    pub text1: Vec<CDataOrText>,
    pub elem: Elem,
    pub text2: String,
}

fn mixed_cdata_separated_1() -> MixedCDataSeparated {
    MixedCDataSeparated {
        text1: vec![
            CDataOrText::String("Text1".to_string()),
            CDataOrText::CData(CData("Text2".to_string())),
        ],
        elem: Elem("Text3".to_string()),
        text2: "Text4".to_string(),
    }
}

define_test!(
    mixed_cdata_separated,
    [(
        mixed_cdata_separated_1(),
        "<mixed>Text1<![CDATA[Text2]]><elem>Text3</elem>Text4</mixed>"
    )]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum NumberOrNoString {
    Number(i32),
    NoString(Option<String>),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "empty_list")]
pub struct EmptyList {
    pub text1: Vec<CDataOrText>,
}

define_test!(
    empty_list,
    [
        (
            EmptyList {
                text1: vec![CDataOrText::String("Text1".to_string())]
            },
            "<empty_list>Text1</empty_list>"
        ),
        (EmptyList { text1: vec![] }, "<empty_list/>"),
        (
            EmptyList { text1: vec![] },
            "<empty_list/>",
            "<empty_list></empty_list>"
        )
    ]
);
