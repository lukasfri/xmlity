use types::top_level_element_items::Child1;

#[derive(
    ::core::fmt::Debug,
    Clone,
    Eq,
    PartialEq,
    ::xmlity::SerializationGroup,
    ::xmlity::DeserializationGroup,
)]
pub struct LocalSimpleType {}

#[derive(
    ::core::fmt::Debug,
    Clone,
    Eq,
    PartialEq,
    ::xmlity::SerializationGroup,
    ::xmlity::DeserializationGroup,
)]
pub struct LocalComplexType {}

pub mod types {
    pub mod top_level_element_items {
        #[derive(
            ::core::fmt::Debug, Clone, Eq, PartialEq, ::xmlity::Serialize, ::xmlity::Deserialize,
        )]
        pub enum Child1 {
            #[xelement(name = "simpleType", namespace = "http://www.w3.org/2001/XMLSchema")]
            SimpleType(#[xgroup] super::super::LocalSimpleType),
            #[xelement(name = "complexType", namespace = "http://www.w3.org/2001/XMLSchema")]
            ComplexType(#[xgroup] super::super::LocalComplexType),
        }
    }

    #[derive(
        ::core::fmt::Debug,
        Clone,
        Eq,
        PartialEq,
        ::xmlity::SerializationGroup,
        ::xmlity::DeserializationGroup,
    )]
    #[xgroup(children_order = "strict")]
    pub struct TopLevelElement {
        #[xattribute(name = "id", optional)]
        pub id: Option<String>,
        #[xattribute(name = "type", optional)]
        pub type_: Option<String>,
        #[xattribute(name = "substitutionGroup", optional)]
        pub substitution_group: Option<String>,
        #[xattribute(name = "default", optional)]
        pub default: Option<String>,
        #[xattribute(name = "fixed", optional)]
        pub fixed: Option<String>,
        #[xattribute(name = "nillable", optional)]
        pub nillable: Option<bool>,
        #[xattribute(name = "abstract", optional)]
        pub abstract_: Option<bool>,
        #[xattribute(name = "final", optional)]
        pub final_: Option<String>,
        #[xattribute(name = "block", optional)]
        pub block: Option<String>,
        #[xattribute(name = "name")]
        pub name: String,
        #[xvalue(default)]
        pub annotation: Option<super::Annotation>,
        #[xvalue(default)]
        pub child_1: Option<top_level_element_items::Child1>,
        #[xelement(
            name = "alternative",
            namespace = "http://www.w3.org/2001/XMLSchema",
            group,
            optional
        )]
        pub alternative: Option<super::AltType>,
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ::xmlity::Serialize, ::xmlity::Deserialize)]
#[xelement(name = "element", namespace = "http://www.w3.org/2001/XMLSchema")]
pub struct TopLevelElement(#[xgroup] pub types::TopLevelElement);

#[derive(Debug, Clone, Eq, PartialEq, ::xmlity::Serialize, ::xmlity::Deserialize)]
#[xelement(name = "annotation", namespace = "http://www.w3.org/2001/XMLSchema")]
pub struct Annotation;
#[derive(
    Debug, Clone, Eq, PartialEq, ::xmlity::SerializationGroup, ::xmlity::DeserializationGroup,
)]
pub struct AltType {}

#[test]
fn test_xml() {
    let xml = r###"
    <xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="anyAttribute" id="anyAttribute">
        <xs:annotation>
            <xs:documentation
                source="../structures/structures.html#element-anyAttribute"/>
            </xs:annotation>
        <xs:complexType>
            <xs:complexContent>
                <xs:extension base="xs:wildcard">
                <xs:attribute name="notQName" type="xs:qnameListA"
                                use="optional"/>
                </xs:extension>
            </xs:complexContent>
        </xs:complexType>
    </xs:element>
    "###
    .trim();

    let value: TopLevelElement = xmlity_quick_xml::de::from_str(xml).unwrap();

    assert_eq!(
        value,
        TopLevelElement(types::TopLevelElement {
            id: Some("anyAttribute".to_string()),
            type_: None,
            substitution_group: None,
            default: None,
            fixed: None,
            nillable: None,
            abstract_: None,
            final_: None,
            block: None,
            name: "anyAttribute".to_string(),
            annotation: Some(Annotation),
            child_1: Some(Child1::ComplexType(LocalComplexType {})),
            alternative: None,
        })
    )
}
