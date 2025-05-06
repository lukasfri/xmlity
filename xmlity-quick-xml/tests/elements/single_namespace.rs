use std::str::FromStr;

use pretty_assertions::assert_eq;

use crate::utils::{
    clean_string, quick_xml_deserialize_test, quick_xml_serialize_test_with_default,
};
use rstest::rstest;
use xmlity::{types::string::Trim, ExpandedName, XmlNamespace};
use xmlity::{Deserialize, Serialize};

const SIMPLE_DEFAULT_NS_1D_STRUCT_TEST_XML: &str = r###"
  <to xmlns="http://my.namespace.example.com/this/is/a/namespace">Tove</to>
"###;

const SIMPLE_DEFAULT_WRONG_NS_1D_STRUCT_TEST_XML: &str = r###"
  <to xmlns="http://not.my.namespace.example.org/this/should/not/match">Tove</to>
"###;

const SIMPLE_NS_1D_STRUCT_TEST_XML: &str = r###"
  <testns:to xmlns:testns="http://my.namespace.example.com/this/is/a/namespace">Tove</testns:to>
"###;

const SIMPLE_WRONG_NS_1D_STRUCT_TEST_XML: &str = r###"
  <testns:to xmlns:testns="http://not.my.namespace.example.org/this/should/not/match">Tove</to>
"###;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(
    name = "to",
    namespace = "http://my.namespace.example.com/this/is/a/namespace"
)]
pub struct To(String);

fn simple_ns_1d_struct() -> To {
    To("Tove".to_string())
}

#[rstest]
#[case::default_ns(
    SIMPLE_DEFAULT_NS_1D_STRUCT_TEST_XML,
    Some("http://my.namespace.example.com/this/is/a/namespace")
)]
#[case::ns(SIMPLE_NS_1D_STRUCT_TEST_XML, None)]
fn simple_ns_1d_struct_serialize(
    #[case] test_xml: &str,
    #[case] default_namespace: Option<&'static str>,
) {
    let actual = quick_xml_serialize_test_with_default(
        simple_ns_1d_struct(),
        default_namespace.map(XmlNamespace::new).map(Result::unwrap),
    )
    .unwrap();

    assert_eq!(actual, clean_string(test_xml));
}

#[rstest]
#[case::default_ns(SIMPLE_DEFAULT_NS_1D_STRUCT_TEST_XML)]
#[case::ns(SIMPLE_NS_1D_STRUCT_TEST_XML)]
fn simple_ns_1d_struct_deserialize(#[case] test_xml: &str) {
    let actual: To = quick_xml_deserialize_test(&clean_string(test_xml)).unwrap();

    let expected = simple_ns_1d_struct();

    assert_eq!(actual, expected);
}

#[rstest]
#[case::default_ns(SIMPLE_DEFAULT_WRONG_NS_1D_STRUCT_TEST_XML)]
#[case::ns(SIMPLE_WRONG_NS_1D_STRUCT_TEST_XML)]
fn simple_ns_1d_struct_wrong_ns_deserialize(#[case] test_xml: &str) {
    let err = quick_xml_deserialize_test::<To>(&clean_string(test_xml))
        .expect_err("deserialization should fail");

    let xmlity_quick_xml::de::Error::WrongName { actual, expected } = err else {
        panic!("unexpected error: {err:?}");
    };

    assert_eq!(
        actual,
        Box::new(ExpandedName::new(
            "to".parse().unwrap(),
            Some(
                XmlNamespace::from_str("http://not.my.namespace.example.org/this/should/not/match")
                    .expect("Valid namespace")
            )
        ))
    );

    assert_eq!(
        expected,
        Box::new(ExpandedName::new(
            "to".parse().unwrap(),
            Some(
                XmlNamespace::from_str("http://my.namespace.example.com/this/is/a/namespace")
                    .expect("Valid namespace")
            )
        ))
    );
}

const SIMPLE_3D_DEFAULT_NS_LIST_TEST_XML: &str = r###"
<breakfast_menu xmlns="http://my.namespace.example.com/this/is/a/namespace">
<food>
    <name>Belgian Waffles</name>
    <price>$5.95</price>
    <description>
   Two of our famous Belgian Waffles with plenty of real maple syrup
   </description>
    <calories>650</calories>
</food>
<food>
    <name>Strawberry Belgian Waffles</name>
    <price>$7.95</price>
    <description>
    Light Belgian waffles covered with strawberries and whipped cream
    </description>
    <calories>900</calories>
</food>
<food>
    <name>Berry-Berry Belgian Waffles</name>
    <price>$8.95</price>
    <description>
    Belgian waffles covered with assorted fresh berries and whipped cream
    </description>
    <calories>900</calories>
</food>
<food>
    <name>French Toast</name>
    <price>$4.50</price>
    <description>
    Thick slices made from our homemade sourdough bread
    </description>
    <calories>600</calories>
</food>
<food>
    <name>Homestyle Breakfast</name>
    <price>$6.95</price>
    <description>
    Two eggs, bacon or sausage, toast, and our ever-popular hash browns
    </description>
    <calories>950</calories>
</food>
</breakfast_menu>
"###;

const SIMPLE_3D_NS_LIST_TEST_XML: &str = r###"
<testns:breakfast_menu xmlns:testns="http://my.namespace.example.com/this/is/a/namespace">
<testns:food>
    <testns:name>Belgian Waffles</testns:name>
    <testns:price>$5.95</testns:price>
    <testns:description>
   Two of our famous Belgian Waffles with plenty of real maple syrup
   </testns:description>
    <testns:calories>650</testns:calories>
</testns:food>
<testns:food>
    <testns:name>Strawberry Belgian Waffles</testns:name>
    <testns:price>$7.95</testns:price>
    <testns:description>
    Light Belgian waffles covered with strawberries and whipped cream
    </testns:description>
    <testns:calories>900</testns:calories>
</testns:food>
<testns:food>
    <testns:name>Berry-Berry Belgian Waffles</testns:name>
    <testns:price>$8.95</testns:price>
    <testns:description>
    Belgian waffles covered with assorted fresh berries and whipped cream
    </testns:description>
    <testns:calories>900</testns:calories>
</testns:food>
<testns:food>
    <testns:name>French Toast</testns:name>
    <testns:price>$4.50</testns:price>
    <testns:description>
    Thick slices made from our homemade sourdough bread
    </testns:description>
    <testns:calories>600</testns:calories>
</testns:food>
<testns:food>
    <testns:name>Homestyle Breakfast</testns:name>
    <testns:price>$6.95</testns:price>
    <testns:description>
    Two eggs, bacon or sausage, toast, and our ever-popular hash browns
    </testns:description>
    <testns:calories>950</testns:calories>
</testns:food>
</testns:breakfast_menu>
"###;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(
    name = "name",
    namespace = "http://my.namespace.example.com/this/is/a/namespace"
)]
pub struct Name(pub String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(
    name = "price",
    namespace = "http://my.namespace.example.com/this/is/a/namespace"
)]
pub struct Price(pub String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(
    name = "description",
    namespace = "http://my.namespace.example.com/this/is/a/namespace"
)]
pub struct Description(pub Trim<String>);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(
    name = "calories",
    namespace = "http://my.namespace.example.com/this/is/a/namespace"
)]
pub struct Calories(pub u16);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(
    name = "food",
    namespace = "http://my.namespace.example.com/this/is/a/namespace"
)]
struct Food {
    name: Name,
    price: Price,
    description: Description,
    calories: Calories,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(
    name = "breakfast_menu",
    namespace = "http://my.namespace.example.com/this/is/a/namespace"
)]
struct BreakfastMenu {
    food: Vec<Food>,
}

fn simple_3d_list_test_value() -> BreakfastMenu {
    BreakfastMenu {
        food: vec![
            Food {
                name: Name("Belgian Waffles".to_string()),
                price: Price("$5.95".to_string()),
                description: Description(Trim(
                    "Two of our famous Belgian Waffles with plenty of real maple syrup".to_string(),
                )),
                calories: Calories(650),
            },
            Food {
                name: Name("Strawberry Belgian Waffles".to_string()),
                price: Price("$7.95".to_string()),
                description: Description(Trim(
                    "Light Belgian waffles covered with strawberries and whipped cream".to_string(),
                )),
                calories: Calories(900),
            },
            Food {
                name: Name("Berry-Berry Belgian Waffles".to_string()),
                price: Price("$8.95".to_string()),
                description: Description(Trim(
                    "Belgian waffles covered with assorted fresh berries and whipped cream"
                        .to_string(),
                )),
                calories: Calories(900),
            },
            Food {
                name: Name("French Toast".to_string()),
                price: Price("$4.50".to_string()),
                description: Description(Trim(
                    "Thick slices made from our homemade sourdough bread".to_string(),
                )),
                calories: Calories(600),
            },
            Food {
                name: Name("Homestyle Breakfast".to_string()),
                price: Price("$6.95".to_string()),
                description: Description(Trim(
                    "Two eggs, bacon or sausage, toast, and our ever-popular hash browns"
                        .to_string(),
                )),
                calories: Calories(950),
            },
        ],
    }
}

#[rstest]
#[case::default_ns(
    SIMPLE_3D_DEFAULT_NS_LIST_TEST_XML,
    Some("http://my.namespace.example.com/this/is/a/namespace")
)]
#[case::no_default_ns(SIMPLE_3D_NS_LIST_TEST_XML, None)]
fn simple_3d_struct_serialize(#[case] xml: &str, #[case] default_ns: Option<&'static str>) {
    let actual = quick_xml_serialize_test_with_default(
        simple_3d_list_test_value(),
        default_ns.map(XmlNamespace::new).map(Result::unwrap),
    )
    .unwrap();
    let expected = clean_string(xml);
    assert_eq!(actual, expected);
}

#[rstest]
#[case::default_ns(SIMPLE_3D_DEFAULT_NS_LIST_TEST_XML)]
#[case::no_default_ns(SIMPLE_3D_NS_LIST_TEST_XML)]
fn simple_3d_struct_deserialize(#[case] xml: &str) {
    let actual: BreakfastMenu = quick_xml_deserialize_test(xml).unwrap();
    let expected = simple_3d_list_test_value();
    assert_eq!(actual, expected);
}
