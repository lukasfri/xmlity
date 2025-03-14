//! Tests for basic functionality. These tests are the most basic and do not include any attributes. They are simply used to test the default behavior of the library.
use pretty_assertions::assert_eq;

mod common;
use common::{clean_string, quick_xml_deserialize_test, quick_xml_serialize_test};

use xmlity::{Deserialize, Serialize};

const SIMPLE_1D_STRUCT_TEST_XML: &str = r###"
  <to>Tove</to>
"###;

const SIMPLE_WRONG_1D_STRUCT_TEST_XML: &str = r###"
  <toa>Tove</toa>
"###;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "to")]
pub struct To(String);

fn simple_1d_struct() -> To {
    To("Tove".to_string())
}

#[test]
fn simple_1d_struct_serialize() {
    let actual = quick_xml_serialize_test(simple_1d_struct()).unwrap();

    assert_eq!(actual, clean_string(SIMPLE_1D_STRUCT_TEST_XML));
}

#[test]
fn simple_1d_struct_deserialize() {
    let actual: To =
        quick_xml_deserialize_test(clean_string(SIMPLE_1D_STRUCT_TEST_XML).as_str()).unwrap();

    let expected = simple_1d_struct();

    assert_eq!(actual, expected);
}

#[test]
fn simple_wrong_1d_struct_deserialize() {
    let actual: Result<To, _> =
        quick_xml_deserialize_test(clean_string(SIMPLE_WRONG_1D_STRUCT_TEST_XML).as_str());
    assert!(matches!(
        actual,
        Err(xmlity_quick_xml::Error::WrongName { .. })
    ));
}

const SIMPLE_2D_STRUCT_TEST_XML: &str = r###"
<note>
  <to>Tove</to>
  <from>Jani</from>
  <heading>Reminder</heading>
  <body>Don&apos;t forget me this weekend!</body>
</note>
"###;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "from")]
pub struct From(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "heading")]
pub struct Heading(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "body")]
pub struct Body(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "note")]
pub struct Note {
    pub to: To,
    pub from: From,
    pub heading: Heading,
    pub body: Body,
}

fn simple_2d_struct_serialize_result() -> Note {
    Note {
        to: To("Tove".to_string()),
        from: From("Jani".to_string()),
        heading: Heading("Reminder".to_string()),
        body: Body("Don't forget me this weekend!".to_string()),
    }
}

fn simple_2d_struct_deserialize_result() -> Note {
    Note {
        to: To("Tove".to_string()),
        from: From("Jani".to_string()),
        heading: Heading("Reminder".to_string()),
        body: Body("Don&apos;t forget me this weekend!".to_string()),
    }
}

#[test]
fn simple_2d_struct_serialize() {
    let actual = quick_xml_serialize_test(simple_2d_struct_serialize_result()).unwrap();

    let expected = clean_string(SIMPLE_2D_STRUCT_TEST_XML);

    assert_eq!(actual, expected);
}

#[test]
fn simple_2d_struct_deserialize() {
    let actual: Note =
        quick_xml_deserialize_test(clean_string(SIMPLE_2D_STRUCT_TEST_XML).as_str()).unwrap();

    let expected = simple_2d_struct_deserialize_result();

    assert_eq!(actual, expected);
}

const SIMPLE_3D_LIST_TEST_XML: &str = r###"
<breakfast_menu>
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "name")]
pub struct Name(pub String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "price")]
pub struct Price(pub String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "description")]
pub struct Description(pub String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "calories")]
pub struct Calories(pub u16);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "food")]
struct Food {
    name: Name,
    price: Price,
    description: Description,
    calories: Calories,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "breakfast_menu")]
struct BreakfastMenu {
    food: Vec<Food>,
}

fn simple_3d_list_test_value() -> BreakfastMenu {
    BreakfastMenu {
        food: vec![
            Food {
                name: Name("Belgian Waffles".to_string()),
                price: Price("$5.95".to_string()),
                description: Description(
                    "Two of our famous Belgian Waffles with plenty of real maple syrup".to_string(),
                ),
                calories: Calories(650),
            },
            Food {
                name: Name("Strawberry Belgian Waffles".to_string()),
                price: Price("$7.95".to_string()),
                description: Description(
                    "Light Belgian waffles covered with strawberries and whipped cream".to_string(),
                ),
                calories: Calories(900),
            },
            Food {
                name: Name("Berry-Berry Belgian Waffles".to_string()),
                price: Price("$8.95".to_string()),
                description: Description(
                    "Belgian waffles covered with assorted fresh berries and whipped cream"
                        .to_string(),
                ),
                calories: Calories(900),
            },
            Food {
                name: Name("French Toast".to_string()),
                price: Price("$4.50".to_string()),
                description: Description(
                    "Thick slices made from our homemade sourdough bread".to_string(),
                ),
                calories: Calories(600),
            },
            Food {
                name: Name("Homestyle Breakfast".to_string()),
                price: Price("$6.95".to_string()),
                description: Description(
                    "Two eggs, bacon or sausage, toast, and our ever-popular hash browns"
                        .to_string(),
                ),
                calories: Calories(950),
            },
        ],
    }
}

#[test]
fn simple_3d_struct_serialize() {
    let actual = quick_xml_serialize_test(simple_3d_list_test_value()).unwrap();

    let expected = clean_string(SIMPLE_3D_LIST_TEST_XML);

    assert_eq!(actual, expected);
}

#[test]
fn simple_3d_struct_deserialize() {
    let actual: BreakfastMenu =
        quick_xml_deserialize_test(clean_string(SIMPLE_3D_LIST_TEST_XML).as_str()).unwrap();

    let expected = simple_3d_list_test_value();

    assert_eq!(actual, expected);
}
