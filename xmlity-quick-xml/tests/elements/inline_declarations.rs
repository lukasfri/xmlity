use crate::define_test;

use xmlity::{DeserializationGroup, Deserialize, SerializationGroup, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "c")]
pub struct C {
    #[xelement(name = "b")]
    pub c: String,
}

define_test!(
    element_with_single_child,
    [(C { c: "A".to_string() }, "<c><b>A</b></c>")]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "d")]
pub struct D {
    #[xelement(name = "b")]
    pub b: String,
    pub c: C,
}

define_test!(
    element_with_multiple_children,
    [
        (
            D {
                b: "A".to_string(),
                c: C { c: "B".to_string() }
            },
            "<d><b>A</b><c><b>B</b></c></d>"
        ),
        (
            vec![
                D {
                    b: "A".to_string(),
                    c: C { c: "B".to_string() }
                },
                D {
                    b: "C".to_string(),
                    c: C { c: "D".to_string() }
                }
            ],
            "<d><b>A</b><c><b>B</b></c></d><d><b>C</b><c><b>D</b></c></d>"
        )
    ]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "e")]
pub struct E {
    pub d: Vec<D>,
}

define_test!(
    element_with_vector_of_children,
    [(
        E {
            d: vec![
                D {
                    b: "A".to_string(),
                    c: C { c: "B".to_string() }
                },
                D {
                    b: "C".to_string(),
                    c: C { c: "D".to_string() }
                }
            ]
        },
        r#"<e><d><b>A</b><c><b>B</b></c></d><d><b>C</b><c><b>D</b></c></d></e>"#
    )]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct F {
    #[xvalue(extendable = "iterator")]
    pub g: Vec<D>,
}

define_test!(
    element_with_extendable,
    [(
        F {
            g: vec![
                D {
                    b: "A".to_string(),
                    c: C { c: "B".to_string() }
                },
                D {
                    b: "C".to_string(),
                    c: C { c: "D".to_string() }
                }
            ]
        },
        r#"<d><b>A</b><c><b>B</b></c></d><d><b>C</b><c><b>D</b></c></d>"#
    )]
);

#[rstest::rstest]
#[case("")]
#[case(r#"<b>A</b><c><b>B</b></c><b>C</b><c><b>D</b></c>"#)]
fn element_with_extendable_wrong_deserialize(#[case] xml: &str) {
    let actual: Result<D, _> = crate::utils::quick_xml_deserialize_test(xml);

    assert!(actual.is_err());
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct H {
    #[xelement(name = "g")]
    pub g: Vec<D>,
}

define_test!(
    multiple_elements,
    [(
        H {
            g: vec![
                D {
                    b: "A".to_string(),
                    c: C { c: "B".to_string() }
                },
                D {
                    b: "C".to_string(),
                    c: C { c: "D".to_string() }
                }
            ]
        },
        r#"<g><d><b>A</b><c><b>B</b></c></d><d><b>C</b><c><b>D</b></c></d></g>"#
    )]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum I {
    #[xelement(name = "g")]
    G(D),
}

define_test!(
    enum_with_inline_element,
    [(
        I::G(D {
            b: "A".to_string(),
            c: C { c: "B".to_string() }
        }),
        r#"<g><d><b>A</b><c><b>B</b></c></d></g>"#
    )]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum J {
    #[xelement(name = "g")]
    G(D),
    U(String),
}

define_test!(
    enum_with_inline_element_or_text,
    [
        (
            I::G(D {
                b: "A".to_string(),
                c: C { c: "B".to_string() }
            }),
            r#"<g><d><b>A</b><c><b>B</b></c></d></g>"#
        ),
        (J::U("A".to_string()), r#"A"#)
    ]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "k")]
pub struct K {
    #[xelement(name = "l", optional)]
    pub l: Option<String>,
}

define_test!(
    element_with_optional_child,
    [
        (
            K {
                l: Some("A".to_string())
            },
            "<k><l>A</l></k>"
        ),
        (K { l: None }, "<k/>", "<k></k>"),
        (K { l: None }, "<k/>")
    ]
);

#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
pub struct N {
    #[xelement(name = "o")]
    pub o: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "M")]
pub struct M {
    #[xelement(name = "n", group)]
    pub n: N,
}

define_test!(
    element_with_group_child,
    [(
        M {
            n: N {
                o: Some("A".to_string())
            }
        },
        "<M><n><o>A</o></n></M>"
    )]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "P")]
pub struct P {
    #[xattribute(name = "q")]
    pub q: String,
}

define_test!(
    element_with_attribute,
    [(P { q: "A".to_string() }, "<P q=\"A\"/>")]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "R")]
pub struct R {
    #[xattribute(name = "s", optional)]
    pub s: Option<String>,
}

define_test!(
    element_with_optional_attribute,
    [
        (
            R {
                s: Some("A".to_string())
            },
            "<R s=\"A\"/>"
        ),
        (R { s: None }, "<R/>")
    ]
);

#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
pub struct S;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(order = "strict")]
pub struct T {
    #[xelement(name = "s", group, optional, default)]
    pub simple_type: Option<Box<S>>,
}

define_test!(
    value_with_optional_group_element,
    [
        (
            T {
                simple_type: Some(Box::new(S))
            },
            "<s/>"
        ),
        (T { simple_type: None }, "")
    ]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "U")]
pub struct U {
    #[xelement(name = "s", group, optional, default)]
    pub simple_type: Option<Box<S>>,
}

define_test!(
    element_with_optional_group_element,
    [
        (
            U {
                simple_type: Some(Box::new(S))
            },
            "<U><s/></U>"
        ),
        (U { simple_type: None }, "<U/>", "<U></U>")
    ]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(order = "strict")]
pub struct V {
    #[xelement(name = "n", group)]
    pub n: Box<N>,
}

define_test!(
    value_with_group_element,
    [(
        V {
            n: Box::new(N {
                o: Some("A".to_string())
            }),
        },
        "<n><o>A</o></n>"
    )]
);

#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
#[xgroup(children_order = "strict")]
pub struct W {
    #[xattribute(name = "a", optional, default)]
    pub a: Option<String>,
    #[xattribute(name = "b", optional, default)]
    pub b: Option<String>,
    #[xattribute(name = "c", optional, default)]
    pub c: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum X {
    #[xelement(
        name = "w",
        namespace = "http://www.w3.org/2001/XMLSchema",
        allow_unknown_attributes = "any"
    )]
    E(#[xgroup] Box<W>),
}

define_test!(
    enum_with_element_attr_group,
    [(
        X::E(Box::new(W {
            a: None,
            b: Some("bvalue".to_string()),
            c: None,
        })),
        r###"<a0:w xmlns:a0="http://www.w3.org/2001/XMLSchema" b="bvalue"/>"###,
        r###"<xs:w xmlns:xs="http://www.w3.org/2001/XMLSchema" d="namespace" b="bvalue"/>"###
    )]
);
