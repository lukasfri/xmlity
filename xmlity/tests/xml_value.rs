#![cfg(feature = "derive")]
use pretty_assertions::assert_eq;

use xmlity::{
    value::{XmlChild, XmlElement, XmlText, XmlValue},
    Deserialize, ExpandedName, LocalName, Serialize,
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "to")]
pub struct To(String);

fn xml_value_1d_struct() -> To {
    To("Tove".to_string())
}

fn xml_value_1d_struct_value() -> XmlValue {
    XmlValue::Element(
        XmlElement::new(ExpandedName::new(LocalName::new("to").unwrap(), None))
            .with_children(vec![XmlChild::Text(XmlText::new("Tove"))]),
    )
}

#[test]
fn xml_value_1d_struct_serialize() {
    let mut actual = XmlValue::None;
    xml_value_1d_struct().serialize(&mut actual).unwrap();
    assert_eq!(actual, xml_value_1d_struct_value());
}

#[test]
fn xml_value_1d_struct_deserialize() {
    let actual = xml_value_1d_struct_value();
    let actual = To::deserialize(&actual).unwrap();
    assert_eq!(actual, xml_value_1d_struct());
}

#[test]
fn xml_value_1d_struct_self_deserialize() {
    let source = xml_value_1d_struct_value();
    let actual = XmlValue::deserialize(&source).unwrap();
    assert_eq!(actual, xml_value_1d_struct_value());
}

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

fn xml_value_2d_struct() -> Note {
    Note {
        to: To("Tove".to_string()),
        from: From("Jani".to_string()),
        heading: Heading("Reminder".to_string()),
        body: Body("Don't forget me this weekend!".to_string()),
    }
}

fn xml_value_2d_struct_value() -> XmlValue {
    XmlValue::Element(
        XmlElement::new(ExpandedName::new(LocalName::new("note").unwrap(), None)).with_children(
            vec![
                XmlElement::new(ExpandedName::new(LocalName::new("to").unwrap(), None))
                    .with_child(XmlChild::Text(XmlText::new("Tove"))),
                XmlElement::new(ExpandedName::new(LocalName::new("from").unwrap(), None))
                    .with_child(XmlText::new("Jani")),
                XmlElement::new(ExpandedName::new(LocalName::new("heading").unwrap(), None))
                    .with_child(XmlText::new("Reminder")),
                XmlElement::new(ExpandedName::new(LocalName::new("body").unwrap(), None))
                    .with_child(XmlText::new("Don't forget me this weekend!")),
            ],
        ),
    )
}

#[test]
fn xml_value_2d_struct_serialize() {
    let mut actual = XmlValue::None;
    xml_value_2d_struct().serialize(&mut actual).unwrap();
    assert_eq!(actual, xml_value_2d_struct_value());
}

#[test]
fn xml_value_2d_struct_deserialize() {
    let actual = xml_value_2d_struct_value();
    let actual = Note::deserialize(&actual).unwrap();
    assert_eq!(actual, xml_value_2d_struct());
}

#[test]
fn xml_value_2d_struct_self_deserialize() {
    let source = xml_value_2d_struct_value();
    let actual = XmlValue::deserialize(&source).unwrap();
    assert_eq!(actual, xml_value_2d_struct_value());
}

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

fn xml_value_3d_list() -> BreakfastMenu {
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

fn xml_value_3d_list_value() -> XmlValue {
    XmlValue::Element(
        XmlElement::new(ExpandedName::new(
            LocalName::new("breakfast_menu").unwrap(),
            None,
        ))
        .with_children(vec![
            XmlElement::new(ExpandedName::new(LocalName::new("food").unwrap(), None))
                .with_children(vec![
                    XmlElement::new(ExpandedName::new(LocalName::new("name").unwrap(), None))
                        .with_child(XmlChild::Text(XmlText::new("Belgian Waffles"))),
                    XmlElement::new(ExpandedName::new(LocalName::new("price").unwrap(), None))
                        .with_child(XmlChild::Text(XmlText::new("$5.95"))),
                    XmlElement::new(ExpandedName::new(
                        LocalName::new("description").unwrap(),
                        None,
                    ))
                    .with_child(XmlChild::Text(XmlText::new(
                        "Two of our famous Belgian Waffles with plenty of real maple syrup",
                    ))),
                    XmlElement::new(ExpandedName::new(LocalName::new("calories").unwrap(), None))
                        .with_child(XmlChild::Text(XmlText::new("650"))),
                ]),
            XmlElement::new(ExpandedName::new(LocalName::new("food").unwrap(), None))
                .with_children(vec![
                    XmlElement::new(ExpandedName::new(LocalName::new("name").unwrap(), None))
                        .with_child(XmlChild::Text(XmlText::new("Strawberry Belgian Waffles"))),
                    XmlElement::new(ExpandedName::new(LocalName::new("price").unwrap(), None))
                        .with_child(XmlChild::Text(XmlText::new("$7.95"))),
                    XmlElement::new(ExpandedName::new(
                        LocalName::new("description").unwrap(),
                        None,
                    ))
                    .with_child(XmlChild::Text(XmlText::new(
                        "Light Belgian waffles covered with strawberries and whipped cream",
                    ))),
                    XmlElement::new(ExpandedName::new(LocalName::new("calories").unwrap(), None))
                        .with_child(XmlChild::Text(XmlText::new("900"))),
                ]),
            XmlElement::new(ExpandedName::new(LocalName::new("food").unwrap(), None))
                .with_children(vec![
                    XmlElement::new(ExpandedName::new(LocalName::new("name").unwrap(), None))
                        .with_child(XmlChild::Text(XmlText::new("Berry-Berry Belgian Waffles"))),
                    XmlElement::new(ExpandedName::new(LocalName::new("price").unwrap(), None))
                        .with_child(XmlChild::Text(XmlText::new("$8.95"))),
                    XmlElement::new(ExpandedName::new(
                        LocalName::new("description").unwrap(),
                        None,
                    ))
                    .with_child(XmlChild::Text(XmlText::new(
                        "Belgian waffles covered with assorted fresh berries and whipped cream",
                    ))),
                    XmlElement::new(ExpandedName::new(LocalName::new("calories").unwrap(), None))
                        .with_child(XmlChild::Text(XmlText::new("900"))),
                ]),
            XmlElement::new(ExpandedName::new(LocalName::new("food").unwrap(), None))
                .with_children(vec![
                    XmlElement::new(ExpandedName::new(LocalName::new("name").unwrap(), None))
                        .with_child(XmlChild::Text(XmlText::new("French Toast"))),
                    XmlElement::new(ExpandedName::new(LocalName::new("price").unwrap(), None))
                        .with_child(XmlChild::Text(XmlText::new("$4.50"))),
                    XmlElement::new(ExpandedName::new(
                        LocalName::new("description").unwrap(),
                        None,
                    ))
                    .with_child(XmlChild::Text(XmlText::new(
                        "Thick slices made from our homemade sourdough bread",
                    ))),
                    XmlElement::new(ExpandedName::new(LocalName::new("calories").unwrap(), None))
                        .with_child(XmlChild::Text(XmlText::new("600"))),
                ]),
            XmlElement::new(ExpandedName::new(LocalName::new("food").unwrap(), None))
                .with_children(vec![
                    XmlElement::new(ExpandedName::new(LocalName::new("name").unwrap(), None))
                        .with_child(XmlChild::Text(XmlText::new("Homestyle Breakfast"))),
                    XmlElement::new(ExpandedName::new(LocalName::new("price").unwrap(), None))
                        .with_child(XmlChild::Text(XmlText::new("$6.95"))),
                    XmlElement::new(ExpandedName::new(
                        LocalName::new("description").unwrap(),
                        None,
                    ))
                    .with_child(XmlChild::Text(XmlText::new(
                        "Two eggs, bacon or sausage, toast, and our ever-popular hash browns",
                    ))),
                    XmlElement::new(ExpandedName::new(LocalName::new("calories").unwrap(), None))
                        .with_child(XmlChild::Text(XmlText::new("950"))),
                ]),
        ]),
    )
}

#[test]
fn xml_value_3d_list_serialize() {
    let mut actual = XmlValue::None;
    xml_value_3d_list().serialize(&mut actual).unwrap();
    assert_eq!(actual, xml_value_3d_list_value());
}

#[test]
fn xml_value_3d_list_deserialize() {
    let actual = xml_value_3d_list_value();
    let actual = BreakfastMenu::deserialize(&actual).unwrap();
    assert_eq!(actual, xml_value_3d_list());
}

#[test]
fn xml_value_3d_list_self_deserialize() {
    let source = xml_value_3d_list_value();
    let actual = XmlValue::deserialize(&source).unwrap();
    assert_eq!(actual, xml_value_3d_list_value());
}
