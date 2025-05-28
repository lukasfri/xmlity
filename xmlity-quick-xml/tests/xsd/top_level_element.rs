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
            #[xelement(name = "simpleType", namespace = "http://www.w3.org/2000/xmlns/")]
            SimpleType(#[xgroup] super::super::LocalSimpleType),
            #[xelement(name = "complexType", namespace = "http://www.w3.org/2000/xmlns/")]
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
        #[xattribute(name = "id", optional, default)]
        pub id: Option<String>,
        #[xattribute(name = "type", optional, default)]
        pub type_: Option<String>,
        #[xattribute(name = "substitutionGroup", optional, default)]
        pub substitution_group: Option<String>,
        #[xattribute(name = "default", optional, default)]
        pub default: Option<String>,
        #[xattribute(name = "fixed", optional, default)]
        pub fixed: Option<String>,
        #[xattribute(name = "nillable", optional, default)]
        pub nillable: Option<bool>,
        #[xattribute(name = "abstract", optional, default)]
        pub abstract_: Option<bool>,
        #[xattribute(name = "final", optional, default)]
        pub final_: Option<String>,
        #[xattribute(name = "block", optional, default)]
        pub block: Option<String>,
        #[xattribute(name = "name")]
        pub name: String,
        #[xvalue(default)]
        pub annotation: Option<super::Annotation>,
        #[xvalue(default)]
        pub child_1: Option<top_level_element_items::Child1>,
        #[xelement(
            name = "alternative",
            namespace = "http://www.w3.org/2000/xmlns/",
            group,
            optional,
            default
        )]
        pub alternative: Option<super::AltType>,
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ::xmlity::Serialize, ::xmlity::Deserialize)]
#[xelement(name = "element", namespace = "http://www.w3.org/2000/xmlns/")]
pub struct TopLevelElement(#[xgroup] pub types::TopLevelElement);

#[test]
fn test_xml() {
    todo!()
}
