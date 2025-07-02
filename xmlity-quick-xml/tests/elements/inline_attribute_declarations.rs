use crate::define_test;

use xmlity::{Deserialize, Serialize};

fn f_serialize<T: xmlity::Serializer>(f: &F, serializer: T) -> Result<T::Ok, T::Error> {
    serializer.serialize_text(&f.0)
}

fn f_deserialize<'de, T: xmlity::Deserializer<'de>>(deserializer: T) -> Result<F, T::Error> {
    let s = String::deserialize(deserializer)?;
    Ok(F(s))
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(serialize_with = "f_serialize", deserialize_with = "f_deserialize")]
struct F(pub String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "c")]
pub struct C {
    #[xattribute(name = "b")]
    pub c: F,
}

define_test!(
    element_with_single_child,
    [(
        C {
            c: F("A".to_string())
        },
        r#"<c b="A"/>"#
    )]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "d")]
pub struct D {
    #[xattribute(name = "b")]
    pub b: String,
    pub c: C,
}

define_test!(
    element_with_multiple_children,
    [(
        D {
            b: "A".to_string(),
            c: C {
                c: F("B".to_string())
            }
        },
        r#"<d b="A"><c b="B"/></d>"#
    )]
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
                    c: C {
                        c: F("B".to_string())
                    }
                },
                D {
                    b: "C".to_string(),
                    c: C {
                        c: F("D".to_string())
                    }
                }
            ]
        },
        r#"<e><d b="A"><c b="B"/></d><d b="C"><c b="D"/></d></e>"#
    )]
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum H {
    F(F),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "g")]
struct G {
    #[xattribute(name = "f", optional, default)]
    pub f: Option<H>,
}

define_test!(
    element_with_optional_attribute_of_enum,
    [
        (
            G {
                f: Some(H::F(F("A".to_string())))
            },
            r#"<g f="A"/>"#
        ),
        (G { f: None }, r#"<g/>"#)
    ]
);
