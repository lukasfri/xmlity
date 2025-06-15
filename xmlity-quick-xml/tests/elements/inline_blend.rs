use xmlity::{DeserializationGroup, Deserialize, SerializationGroup, Serialize};

use crate::define_test;

#[derive(Debug, Clone, Eq, PartialEq, SerializationGroup, DeserializationGroup)]
#[xgroup(children_order = "strict")]
pub struct AGroup {
    #[xattribute(name = "attr-a", optional)]
    pub attr_a: Option<String>,
    #[xattribute(name = "attr-b")]
    pub attr_b: String,
    #[xvalue(default)]
    pub b: Option<B>,
    #[xelement(name = "c", group, optional)]
    pub c: Option<C>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[xelement(name = "a")]
pub struct A(#[xgroup] pub AGroup);

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[xelement(name = "a", children_order = "strict")]
pub struct ATop {
    #[xattribute(name = "attr-a", optional)]
    pub attr_a: Option<String>,
    #[xattribute(name = "attr-b")]
    pub attr_b: String,
    #[xvalue(default)]
    pub b: Option<B>,
    #[xelement(name = "c", group, optional)]
    pub c: Option<C>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[xelement(name = "b")]
pub struct B;

#[derive(Debug, Clone, Eq, PartialEq, SerializationGroup, DeserializationGroup)]
pub struct C {}

define_test!(
    group_element,
    [(
        A(AGroup {
            attr_a: Some("AVal".to_string()),
            attr_b: "BVal".to_string(),
            b: Some(B),
            c: None,
        }),
        r###"<a attr-a="AVal" attr-b="BVal"><b/></a>"###,
        r###"
        <a attr-b="BVal" attr-a="AVal">
            <b>
                <whatever source="does not matter"/>
            </b>
        </a>
        "###
        .trim()
    )]
);

define_test!(
    top_element,
    [(
        ATop {
            attr_a: Some("AVal".to_string()),
            attr_b: "BVal".to_string(),
            b: Some(B),
            c: None,
        },
        r###"<a attr-a="AVal" attr-b="BVal"><b/></a>"###,
        r###"
        <a attr-b="BVal" attr-a="AVal">
            <b>
                <whatever source="does not matter"/>
            </b>
        </a>
        "###
        .trim()
    )]
);
