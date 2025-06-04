use xmlity::{DeserializationGroup, Deserialize, SerializationGroup, Serialize};

use crate::define_test;

#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
pub struct BlockGroup(#[xvalue(default)] pub Vec<String>);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "block")]
pub struct Block(#[xgroup] pub BlockGroup);

define_test!(
    unnamed_group,
    [(
        Block(BlockGroup(vec!["Hello World".to_string()])),
        "<block>Hello World</block>"
    )]
);
