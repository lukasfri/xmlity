use xmlity::{Deserialize, Serialize};

use crate::define_deserialize_test;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum XhtmlBlockClass {
    XhtmlBlockExtra(Box<XhtmlBlockExtra>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[xvalue(order = "strict")]
pub struct XhtmlBlockExtra;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum XhtmlBlockMix {
    XhtmlBlockClass(Box<XhtmlBlockClass>),
}

const XHTML_BODY_CONTENT: &str = r###"

  some content here...

"###;

define_deserialize_test!(
    test_not_stopping,
    [(Vec::<XhtmlBlockMix>::new(), XHTML_BODY_CONTENT)]
);
