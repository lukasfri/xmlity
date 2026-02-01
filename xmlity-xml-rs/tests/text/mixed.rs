use crate::define_test;

use xmlity::types::utils::CData;
use xmlity::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MixedCData {
    pub text1: String,
    pub cdata: CData<String>,
    pub text2: String,
}

define_test!(
    mixed_cdata_struct,
    [(
        MixedCData {
            text1: "Text".to_string(),
            cdata: CData("More".to_string()),
            text2: "Text2".to_string(),
        },
        "Text<![CDATA[More]]>Text2"
    )]
);
