use pretty_assertions::assert_eq;

use crate::define_test;
use crate::utils::{clean_string, quick_xml_deserialize_test, quick_xml_serialize_test};

use rstest::rstest;
use xmlity::{
    DeserializationGroup, Deserialize, SerializationGroup, Serialize, SerializeAttribute,
};

const SIMPLE_2D_STRUCT_TEST_XML: &str = r###"
<note to="Tove" from="Jani">
  <heading>Reminder</heading>
  <body>Don't forget me this weekend!</body>
</note>
"###;

const SIMPLE_2D_STRUCT_TEST_XML_WRONG_ORDER: &str = r###"
<note to="Tove" from="Jani">
  <body>Don't forget me this weekend!</body>
  <heading>Reminder</heading>
</note>
"###;

#[derive(Debug, PartialEq, SerializeAttribute, Deserialize)]
#[xattribute(name = "to")]
pub struct To(String);

#[derive(Debug, PartialEq, SerializeAttribute, Deserialize)]
#[xattribute(name = "from")]
pub struct From(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "heading")]
pub struct Heading(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "body")]
pub struct Body(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "note", attribute_order = "strict", children_order = "strict")]
pub struct Note {
    #[xattribute(deferred = true)]
    pub to: To,
    #[xattribute(deferred = true)]
    pub from: From,
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

define_test!(
    struct_2d_with_attributes,
    [(
        simple_2d_struct_result(),
        clean_string(SIMPLE_2D_STRUCT_TEST_XML)
    )]
);

#[test]
fn struct_2d_with_attributes_deserialize_fail() {
    let actual: Result<Note, _> =
        quick_xml_deserialize_test(clean_string(SIMPLE_2D_STRUCT_TEST_XML_WRONG_ORDER).as_str());

    assert!(actual.is_err());
    // match actual.unwrap_err() {
    //     xmlity_quick_xml::de::Error::WrongName { actual, expected } => {
    //         assert_eq!(
    //             *actual,
    //             ExpandedName::new(LocalName::new("body").unwrap(), None)
    //         );
    //         assert_eq!(
    //             *expected,
    //             ExpandedName::new(LocalName::new("heading").unwrap(), None)
    //         );
    //     }
    //     e => panic!("Wrong error type: {e:?}"),
    // };
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "type")]
pub struct HammerType(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "name")]
pub struct ToolName(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "weight")]
pub struct Weight(u32);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "shape")]
pub struct HammerShape(String);

#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
#[xgroup]
pub struct Tool {
    #[xvalue]
    pub name: ToolName,
    #[xvalue]
    pub weight: Weight,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "hammer", attribute_order = "strict", children_order = "strict")]
pub struct Hammer {
    pub hammer_type: HammerType,
    #[xgroup]
    pub tool: Tool,
    pub shape: HammerShape,
}

const STRUCT_WITH_GROUP_ORDER_EXACT_ORDER: &str = r#"
<hammer>
    <type>Hammer</type>
    <name>Hammer</name>
    <weight>10</weight>
    <shape>Square</shape>
</hammer>
"#;

const STRUCT_WITH_GROUP_ORDER_OK_REORDER: &str = r#"
<hammer>
    <type>Hammer</type>
    <weight>10</weight>
    <name>Hammer</name>
    <shape>Square</shape>
</hammer>
"#;

const STRUCT_WITH_GROUP_ORDER_TEST_XML_WRONG_ORDER1: &str = r#"
<hammer>
    <type>Hammer</type>
    <weight>10</weight>
    <shape>Square</shape>
    <name>Hammer</name>
</hammer>
"#;

const STRUCT_WITH_GROUP_ORDER_TEST_XML_WRONG_ORDER2: &str = r#"
<hammer>
    <weight>10</weight>
    <type>Hammer</type>
    <name>Hammer</name>
    <shape>Square</shape>
</hammer>
"#;

const STRUCT_WITH_GROUP_ORDER_TEST_XML_WRONG_INCOMPLETE_GROUP: &str = r#"
<hammer>
    <weight>10</weight>
    <name>Hammer</name>
    <shape>Square</shape>
</hammer>
"#;

fn hammer_struct_result() -> Hammer {
    Hammer {
        hammer_type: HammerType("Hammer".to_string()),
        tool: Tool {
            name: ToolName("Hammer".to_string()),
            weight: Weight(10),
        },
        shape: HammerShape("Square".to_string()),
    }
}

define_test!(
    struct_with_group_order,
    [
        (
            hammer_struct_result(),
            clean_string(STRUCT_WITH_GROUP_ORDER_EXACT_ORDER)
        ),
        (
            hammer_struct_result(),
            clean_string(STRUCT_WITH_GROUP_ORDER_EXACT_ORDER),
            clean_string(STRUCT_WITH_GROUP_ORDER_OK_REORDER)
        )
    ]
);

#[rstest]
#[case(STRUCT_WITH_GROUP_ORDER_TEST_XML_WRONG_ORDER1)]
#[case(STRUCT_WITH_GROUP_ORDER_TEST_XML_WRONG_ORDER2)]
#[case(STRUCT_WITH_GROUP_ORDER_TEST_XML_WRONG_INCOMPLETE_GROUP)]
fn struct_with_group_order_deserialize_fail(#[case] xml: &str) {
    let actual: Result<Hammer, _> = quick_xml_deserialize_test(clean_string(xml).as_str());

    assert!(actual.is_err());
    //TODO: assert error type
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "type")]
pub struct CarType(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "name")]
pub struct VehicleName(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "shape")]
pub struct CarShape(String);

#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
#[xgroup(attribute_order = "strict", children_order = "strict")]
pub struct Vehicle {
    #[xvalue]
    pub name: VehicleName,
    #[xvalue]
    pub weight: Weight,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "car", attribute_order = "strict", children_order = "strict")]
pub struct Car {
    pub car_type: CarType,
    #[xgroup]
    pub vehicle: Vehicle,
    pub shape: CarShape,
}

const CAR_WITH_GROUP_ORDER_EXACT_ORDER: &str = r#"
<car>
    <type>Car</type>
    <name>Car</name>
    <weight>10</weight>
    <shape>Square</shape>
</car>
"#;

const CAR_WITH_GROUP_ORDER_TEST_XML_WRONG_ORDER1: &str = r#"
<car>
    <type>Car</type>
    <weight>10</weight>
    <shape>Square</shape>
    <name>Car</name>
</car>
"#;

const CAR_WITH_GROUP_ORDER_TEST_XML_WRONG_ORDER2: &str = r#"
<car>
    <weight>10</weight>
    <type>Car</type>
    <name>Car</name>
    <shape>Square</shape>
</car>
"#;

const CAR_WITH_GROUP_ORDER_TEST_XML_WRONG_ORDER3: &str = r#"
<car>
    <type>Car</type>
    <weight>10</weight>
    <name>Car</name>
    <shape>Square</shape>
</car>
"#;

const CAR_WITH_GROUP_ORDER_TEST_XML_WRONG_INCOMPLETE_GROUP: &str = r#"
<car>
    <weight>10</weight>
    <name>Car</name>
    <shape>Square</shape>
</car>
"#;

fn car_struct_result() -> Car {
    Car {
        car_type: CarType("Car".to_string()),
        vehicle: Vehicle {
            name: VehicleName("Car".to_string()),
            weight: Weight(10),
        },
        shape: CarShape("Square".to_string()),
    }
}

#[test]
fn car_with_group_order_serialize() {
    let car = car_struct_result();

    let actual = quick_xml_serialize_test(car).unwrap();

    let expected = clean_string(CAR_WITH_GROUP_ORDER_EXACT_ORDER);

    assert_eq!(actual, expected);
}

#[rstest]
#[case(CAR_WITH_GROUP_ORDER_EXACT_ORDER)]
fn car_with_group_order_deserialize(#[case] xml: &str) {
    let actual: Car = quick_xml_deserialize_test(clean_string(xml).as_str()).unwrap();

    let expected = car_struct_result();

    assert_eq!(actual, expected);
}

#[rstest]
#[case(CAR_WITH_GROUP_ORDER_TEST_XML_WRONG_ORDER1)]
#[case(CAR_WITH_GROUP_ORDER_TEST_XML_WRONG_ORDER2)]
#[case(CAR_WITH_GROUP_ORDER_TEST_XML_WRONG_ORDER3)]
#[case(CAR_WITH_GROUP_ORDER_TEST_XML_WRONG_INCOMPLETE_GROUP)]
fn car_with_group_order_deserialize_fail(#[case] xml: &str) {
    let actual: Result<Car, _> = quick_xml_deserialize_test(clean_string(xml).as_str());

    assert!(actual.is_err());
    //TODO: assert error type
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "type")]
pub struct ClothingType(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "name")]
pub struct ClothingName(String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "shape")]
pub struct ClothingShape(String);

#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
#[xgroup(attribute_order = "strict", children_order = "strict")]
pub struct Clothing {
    #[xvalue]
    pub name: ClothingName,
    #[xvalue]
    pub weight: Weight,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "shirt")]
pub struct Shirt {
    pub clothing_type: ClothingType,
    #[xgroup]
    pub clothing: Clothing,
    pub shape: ClothingShape,
}

const SHIRT_WITH_GROUP_ORDER_EXACT_ORDER: &str = r#"
<shirt>
    <type>Shirt</type>
    <name>Shirt</name>
    <weight>10</weight>
    <shape>Square</shape>
</shirt>
"#;

const SHIRT_WITH_GROUP_ORDER_OK_REORDER1: &str = r#"
<shirt>
    <name>Shirt</name>
    <weight>10</weight>
    <type>Shirt</type>
    <shape>Square</shape>
</shirt>
"#;

const SHIRT_WITH_GROUP_ORDER_OK_REORDER2: &str = r#"
<shirt>
    <shape>Square</shape>
    <type>Shirt</type>
    <name>Shirt</name>
    <weight>10</weight>
</shirt>
"#;

const SHIRT_WITH_GROUP_ORDER_TEST_XML_WRONG_ORDER1: &str = r#"
<shirt>
    <type>Shirt</type>
    <weight>10</weight>
    <shape>Square</shape>
    <name>Shirt</name>
</shirt>
"#;

const SHIRT_WITH_GROUP_ORDER_TEST_XML_WRONG_ORDER2: &str = r#"
<shirt>
    <weight>10</weight>
    <type>Shirt</type>
    <name>Shirt</name>
    <shape>Square</shape>
</shirt>
"#;

const SHIRT_WITH_GROUP_ORDER_TEST_XML_WRONG_ORDER3: &str = r#"
<shirt>
    <type>Shirt</type>
    <weight>10</weight>
    <name>Shirt</name>
    <shape>Square</shape>
</shirt>
"#;

const SHIRT_WITH_GROUP_ORDER_TEST_XML_WRONG_INCOMPLETE_GROUP: &str = r#"
<shirt>
    <weight>10</weight>
    <name>Shirt</name>
    <shape>Square</shape>
</shirt>
"#;

fn shirt_struct_result() -> Shirt {
    Shirt {
        clothing_type: ClothingType("Shirt".to_string()),
        clothing: Clothing {
            name: ClothingName("Shirt".to_string()),
            weight: Weight(10),
        },
        shape: ClothingShape("Square".to_string()),
    }
}

#[test]
fn shirt_with_group_order_serialize() {
    let shirt = shirt_struct_result();

    let actual = quick_xml_serialize_test(shirt).unwrap();

    let expected = clean_string(SHIRT_WITH_GROUP_ORDER_EXACT_ORDER);

    assert_eq!(actual, expected);
}

#[rstest]
#[case(SHIRT_WITH_GROUP_ORDER_EXACT_ORDER)]
#[case(SHIRT_WITH_GROUP_ORDER_OK_REORDER1)]
#[case(SHIRT_WITH_GROUP_ORDER_OK_REORDER2)]
fn shirt_with_group_order_deserialize(#[case] xml: &str) {
    let actual: Shirt = quick_xml_deserialize_test(clean_string(xml).as_str()).unwrap();

    let expected = shirt_struct_result();

    assert_eq!(actual, expected);
}

#[rstest]
#[case(SHIRT_WITH_GROUP_ORDER_TEST_XML_WRONG_ORDER1)]
#[case(SHIRT_WITH_GROUP_ORDER_TEST_XML_WRONG_ORDER2)]
#[case(SHIRT_WITH_GROUP_ORDER_TEST_XML_WRONG_ORDER3)]
#[case(SHIRT_WITH_GROUP_ORDER_TEST_XML_WRONG_INCOMPLETE_GROUP)]
fn shirt_with_group_order_deserialize_fail(#[case] xml: &str) {
    let actual: Result<Shirt, _> = quick_xml_deserialize_test(clean_string(xml).as_str());

    assert!(actual.is_err());
    //TODO: assert error type
}
