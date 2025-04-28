//! Tests for basic functionality. These tests are the most basic and do not include any attributes. They are simply used to test the default behavior of the library.
use pretty_assertions::assert_eq;

use crate::utils::{clean_string, quick_xml_deserialize_test, quick_xml_serialize_test};

use xmlity::{Deserialize, Serialize, SerializeAttribute};

const SIMPLE_2D_STRUCT_TEST_XML: &str = r###"
<note to="Tove" from="Jani" heading="Reminder">
  <body>Don't forget me this weekend!</body>
</note>
"###;

#[derive(Debug, PartialEq, SerializeAttribute, Deserialize)]
#[xattribute(name = "to")]
pub struct To(String);

#[derive(Debug, PartialEq, SerializeAttribute, Deserialize)]
#[xattribute(name = "from")]
pub struct From(String);

#[derive(Debug, PartialEq, SerializeAttribute, Deserialize)]
#[xattribute(name = "heading")]
pub struct Heading(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "body")]
pub struct Body(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "note")]
pub struct Note {
    #[xattribute]
    pub to: To,
    #[xattribute]
    pub from: From,
    #[xattribute]
    pub heading: Heading,
    pub body: Body,
}

fn simple_2d_struct_result() -> Note {
    Note {
        to: To("Tove".to_string()),
        from: From("Jani".to_string()),
        heading: Heading("Reminder".to_string()),
        body: Body("Don't forget me this weekend!".to_string()),
    }
}

#[test]
fn struct_2d_with_attributes_serialize() {
    let actual = quick_xml_serialize_test(simple_2d_struct_result()).unwrap();

    let expected = clean_string(SIMPLE_2D_STRUCT_TEST_XML);

    assert_eq!(actual, expected);
}

#[test]
fn struct_2d_with_attributes_deserialize() {
    let actual: Note =
        quick_xml_deserialize_test(clean_string(SIMPLE_2D_STRUCT_TEST_XML).as_str()).unwrap();

    let expected = simple_2d_struct_result();

    assert_eq!(actual, expected);
}

const SIMPLE_3D_LIST_TEST_XML: &str = r###"
<breakfast_menu>
<food name="Belgian Waffles" price="$5.95" calories="650">
    <description>
   Two of our famous Belgian Waffles with plenty of real maple syrup
   </description>
</food>
<food name="Strawberry Belgian Waffles" price="$7.95" calories="900">
    <description>
    Light Belgian waffles covered with strawberries and whipped cream
    </description>
</food>
<food name="Berry-Berry Belgian Waffles" price="$8.95" calories="900">
    <description>
    Belgian waffles covered with assorted fresh berries and whipped cream
    </description>
</food>
<food name="French Toast" price="$4.50" calories="600">
    <description>
    Thick slices made from our homemade sourdough bread
    </description>
</food>
<food name="Homestyle Breakfast" price="$6.95" calories="950">
    <description>
    Two eggs, bacon or sausage, toast, and our ever-popular hash browns
    </description>
</food>
</breakfast_menu>
"###;

#[derive(Debug, PartialEq, SerializeAttribute, Deserialize)]
#[xattribute(name = "name")]
pub struct Name(pub String);

#[derive(Debug, PartialEq, SerializeAttribute, Deserialize)]
#[xattribute(name = "price")]
pub struct Price(pub String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "description")]
pub struct Description(pub String);

#[derive(Debug, PartialEq, SerializeAttribute, Deserialize)]
#[xattribute(name = "calories")]
pub struct Calories(pub u16);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "food")]
struct Food {
    #[xattribute]
    name: Name,
    #[xattribute]
    price: Price,
    description: Description,
    #[xattribute]
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
fn struct_3d_with_attributes_serialize() {
    let actual = quick_xml_serialize_test(simple_3d_list_test_value()).unwrap();

    let expected = clean_string(SIMPLE_3D_LIST_TEST_XML);

    assert_eq!(actual, expected);
}

#[test]
fn struct_3d_with_attributes_deserialize() {
    let actual: BreakfastMenu =
        quick_xml_deserialize_test(clean_string(SIMPLE_3D_LIST_TEST_XML).as_str()).unwrap();

    let expected = simple_3d_list_test_value();

    assert_eq!(actual, expected);
}
