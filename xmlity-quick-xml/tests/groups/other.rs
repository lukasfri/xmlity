use std::str::FromStr;

use pretty_assertions::assert_eq;
use xmlity::{
    value::XmlText, DeserializationGroup, Deserialize, LocalNameBuf, SerializationGroup, Serialize,
    XmlValue,
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[xvalue(order = "strict")]
pub struct XmlValueDocChild {
    pub child_0: XmlValue,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[xelement(name = "i", allow_unknown_attributes = "any")]
pub struct I {
    #[xattribute(name = "source", optional, default)]
    pub source: Option<String>,

    pub particle: Vec<XmlValueDocChild>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[xelement(name = "c", allow_unknown_attributes = "any")]
pub struct C {
    #[xattribute(name = "id", optional, default)]
    pub id: Option<String>,
    #[xvalue(default)]
    pub annotation: Vec<I>,
}

#[derive(Debug, SerializationGroup, DeserializationGroup, PartialEq)]
#[xgroup(children_order = "strict")]
pub struct FGroup {
    #[xattribute(name = "name", optional, default)]
    pub name: Option<LocalNameBuf>,
    #[xattribute(name = "type", optional, default)]
    pub type_: Option<String>,
    #[xvalue(default)]
    pub c: Option<Box<C>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum E {
    #[xelement(name = "f", allow_unknown_attributes = "any")]
    F(#[xgroup] Box<FGroup>),
    #[xelement(name = "g", allow_unknown_attributes = "any")]
    G,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[xvalue(order = "strict")]
pub struct H {
    #[xvalue(default)]
    pub attribute: Vec<E>,
    #[xvalue(default)]
    pub any_attribute: Option<Box<D>>,
}

#[derive(Debug, SerializationGroup, DeserializationGroup, PartialEq)]
#[xgroup(children_order = "strict")]
pub struct AGroup {
    #[xattribute(name = "id", optional, default)]
    pub id: Option<String>,
    #[xattribute(name = "base")]
    pub base: String,
    #[xvalue(default)]
    pub b: Option<B>,
    pub h: Box<H>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[xelement(name = "b", allow_unknown_attributes = "any")]
pub struct B;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[xelement(name = "d", allow_unknown_attributes = "any")]
pub struct D;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[xelement(name = "a", allow_unknown_attributes = "any")]
pub struct A(#[xgroup] AGroup);

const TOTAL: &str = r###"
<a base="annotated">
    <b/>
    <f name="mixed" type="boolean">
        <c>
            <i>IValue</i>
        </c>
    </f>
</a>
"###;

#[test]
#[ntest::timeout(1000)]
fn total_test() {
    println!("Start.");

    let xml = TOTAL.trim();
    let actual: A = xmlity_quick_xml::de::from_str(xml).unwrap();

    let expected = A(AGroup {
        id: None,
        base: "annotated".to_string(),
        b: Some(B),
        h: Box::new(H {
            attribute: vec![E::F(Box::new(FGroup {
                name: Some(LocalNameBuf::from_str("mixed").unwrap()),
                type_: Some("boolean".to_string()),
                c: Some(Box::new(C {
                    id: None,
                    annotation: vec![I {
                        source: None,
                        particle: vec![XmlValueDocChild {
                            child_0: XmlValue::Text(XmlText::new("IValue")),
                        }],
                    }],
                })),
            }))],
            any_attribute: None,
        }),
    });

    assert_eq!(actual, expected);

    println!("Done.");
}

const COMPACT: &str = r###"
<i>IValue</i>
"###;

#[test]
#[ntest::timeout(10)]
fn compact_test() {
    println!("Start.");

    let xml = COMPACT.trim();
    let actual: I = xmlity_quick_xml::de::from_str(xml).unwrap();

    let expected = I {
        source: None,
        particle: vec![XmlValueDocChild {
            child_0: XmlValue::Text(XmlText::new("IValue")),
        }],
    };

    assert_eq!(actual, expected);

    println!("Done.");
}
