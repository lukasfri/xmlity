use xmlity::{value::XmlText, Deserialize, Serialize, XmlValue};

use crate::define_deserialize_test;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[xelement(name = "a", namespace = "http://example.com/test")]
pub struct A {
    #[xvalue(default)]
    pub values: Vec<XmlValue>,
}
define_deserialize_test!(
    a,
    [(
        A {
            values: vec![xmlity::xml!(
                <"b":"http://example.com/test">["\n\t\t"
                    <"c":"http://example.com/test">["Fixed Assets"]</"c">"\n\t\t"
                    <"c":"http://example.com/test">["Change in Retained Earnings"]</"c">"\n\t"
                ]</"b">
            )
            .into()],
        },
        XML.trim()
    )]
);

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

const XML: &str = r###"
<a xmlns="http://example.com/test">
	<b>
		<c>Fixed Assets</c>
		<c>Change in Retained Earnings</c>
	</b>
</a>
"###;

#[test]
fn indirect() {
    let a =
        xmlity_quick_xml::from_str::<A>(XML.trim()).expect("Failed to parse schema from string");

    let b_values = a
        .values
        .iter()
        .map(|a| B::deserialize(a).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(
        b_values,
        vec![
            B {
                c: vec![C {
                    value: XmlText::new("Fixed Assets").into()
                }],
            },
            B {
                c: vec![C {
                    value: XmlText::new("Change in Retained Earnings").into()
                }],
            },
        ]
    );
}
