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
        ("Text", CDataOrText::String("Text".to_owned())),
        (
            "<![CDATA[CData]]>",
            CDataOrText::CData(CData("CData".to_owned()))
        )
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
    [("<list>Text1<![CDATA[Text2]]></list>", mixed_cdata_list())]
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
        "<mixed>Text1<![CDATA[Text2]]><elem>Text3</elem>Text4</mixed>",
        mixed_cdata_separated_1()
    )]
);
