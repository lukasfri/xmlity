use xmlity::{Deserialize, Serialize, XmlValue};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[xelement(
    name = "b",
    namespace = "http://example.com/test",
    allow_unknown_attributes = "any"
)]
pub struct B {
    #[xvalue(default)]
    pub c: Vec<C>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[xelement(
    name = "c",
    namespace = "http://example.com/test",
    allow_unknown_attributes = "any"
)]
#[xgroup(children_order = "strict")]
pub struct C {
    pub value: XmlValue,
}

#[test]
fn value() {
    let value = xmlity::xml!(<"b":"http://example.com/test">["\n\t\t"
        <"c":"http://example.com/test">["Fixed Assets"]</"c">"\n\t\t"
        <"c":"http://example.com/test">["Change in Retained Earnings"]</"c">"\n\t"
    ]</"b">);

    let b = B::deserialize(&value).unwrap();

    assert_eq!(
        b,
        B {
            c: vec![
                C {
                    value: XmlValue::Text("Fixed Assets".into())
                },
                C {
                    value: XmlValue::Text("Change in Retained Earnings".into())
                },
            ],
        }
    )
}
