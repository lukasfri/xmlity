use xmlity::{DeserializationGroup, Deserialize, SerializationGroup, Serialize};

use crate::define_test;

#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
pub struct BlockGroup;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "block")]
pub struct Block(#[xgroup] pub BlockGroup);

define_test!(unnamed_group, [(Block(BlockGroup), "<block/>")]);
