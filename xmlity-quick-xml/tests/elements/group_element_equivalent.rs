use xmlity::{
    DeserializationGroup, Deserialize, LocalName, SerializationGroup, Serialize, XmlNamespace,
};

#[test]
fn complex_type_alike_wrapped_element_equals_element() {
    const XML: &str = r###"
    <xs:a xmlns:xs="http://www.w3.org/2001/XMLSchema">
      <xs:d>
        <xs:e/>
        <xs:c/>
        <xs:c/>
        <xs:c/>
      </xs:d>
    </xs:a>
    "###;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[xelement(name = "b", namespace_expr = XmlNamespace::XS)]
    struct B;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[xelement(name = "c", namespace_expr = XmlNamespace::XS)]
    struct C;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[xelement(name = "e", namespace_expr = XmlNamespace::XS)]
    struct E;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[xelement(name = "f", namespace_expr = XmlNamespace::XS)]
    struct F;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum Enum2 {
        Other {
            #[xvalue(default)]
            e: Option<E>,
            #[xvalue(default)]
            c: Vec<C>,
        },
    }

    #[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
    struct D {
        #[xvalue(default)]
        pub annotation: Option<B>,
        pub content: Enum2,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum Enum1 {
        #[xelement(name = "d", namespace = "http://www.w3.org/2001/XMLSchema")]
        D(#[xgroup] D),
    }

    #[derive(
        Debug, PartialEq, Serialize, Deserialize, SerializationGroup, DeserializationGroup,
    )]
    #[xelement(name = "a", namespace_expr = XmlNamespace::XS, children_order = "loose")]
    #[xgroup(children_order = "strict")]
    pub struct A {
        #[xvalue(default)]
        pub b: Option<B>,
        #[xvalue(default)]
        pub enum_1: Option<Enum1>,
        #[xvalue(default)]
        pub c: Option<C>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[xelement(name = "a", namespace_expr = XmlNamespace::XS)]
    pub struct AWrapped(#[xgroup] pub A);

    let a: A = xmlity_quick_xml::de::from_str(XML.trim()).unwrap();

    let a_wrapped: AWrapped = xmlity_quick_xml::de::from_str(XML.trim()).unwrap();

    let a_expected = A {
        b: None,
        enum_1: Some(Enum1::D(D {
            annotation: None,
            content: Enum2::Other {
                e: Some(E),
                c: vec![C, C, C],
            },
        })),
        c: None,
    };

    assert_eq!(a_expected, a);

    assert_eq!(a_expected, a_wrapped.0);
}

#[test]
fn simple_wrapped_element_equals_element() {
    const XML: &str = r###"
    <xs:a xmlns:xs="http://www.w3.org/2001/XMLSchema">
      <xs:c/>
    </xs:a>
    "###;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[xelement(name = "b", namespace_expr = XmlNamespace::XS)]
    struct B;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[xelement(name = "c", namespace_expr = XmlNamespace::XS)]
    struct C;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[xelement(name = "d", namespace_expr = XmlNamespace::XS)]
    struct D;

    #[derive(
        Debug, PartialEq, Serialize, Deserialize, SerializationGroup, DeserializationGroup,
    )]
    #[xelement(name = "a", namespace_expr = XmlNamespace::XS, children_order = "loose")]
    #[xgroup(children_order = "strict")]
    pub struct A {
        #[xvalue(default)]
        pub b: Option<B>,
        #[xvalue(default)]
        pub c: Option<C>,
        #[xvalue(default)]
        pub d: Option<D>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[xelement(name = "a", namespace_expr = XmlNamespace::XS)]
    pub struct AWrapped(#[xgroup] pub A);

    let a: A = xmlity_quick_xml::de::from_str(XML.trim()).unwrap();

    let a_wrapped: AWrapped = xmlity_quick_xml::de::from_str(XML.trim()).unwrap();

    let a_expected = A {
        b: None,
        c: Some(C),
        d: None,
    };

    assert_eq!(a_expected, a);

    assert_eq!(a_expected, a_wrapped.0);
}

#[test]
fn top_level_xs_wrapped_element_equals_element() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[xelement(
        name = "attributeGroup",
        namespace = "http://www.w3.org/2001/XMLSchema"
    )]
    struct AttributeGroupRefType {
        #[xattribute(name = "ref", optional)]
        pub ref_: Option<String>,
        #[xvalue(default)]
        pub annotation: Option<Anno>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[xelement(name = "element", namespace = "http://www.w3.org/2001/XMLSchema")]
    struct LocalElement {
        #[xattribute(name = "ref", optional)]
        pub ref_: Option<String>,
        #[xattribute(name = "maxOccurs", optional)]
        pub max_occurs: Option<String>,
        #[xvalue(default)]
        pub annotation: Option<Anno>,
        // #[xvalue(default)]
        // pub type_choice: Option<types::top_level_element_items::Child1>,
        #[xvalue(default)]
        pub alternatives: Vec<Alt>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum GroupTypeContent {
        Element(LocalElement),
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[xelement(name = "sequence", namespace = "http://www.w3.org/2001/XMLSchema")]
    struct SequenceType {
        pub content: Vec<LocalElement>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum TypeDefParticle {
        Sequence(SequenceType),
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum ComplexTypeModel {
        Other {
            #[xvalue(default)]
            open_content: Option<OpenContent>,
            #[xvalue(default)]
            type_def_particle: Option<TypeDefParticle>,
            #[xvalue(default)]
            attributes: Vec<AttributeGroupRefType>,
        },
    }

    #[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
    struct LocalComplexType {
        #[xvalue(default)]
        pub annotation: Option<Anno>,
        pub content: ComplexTypeModel,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum Child1 {
        #[xelement(name = "complexType", namespace = "http://www.w3.org/2001/XMLSchema")]
        ComplexType(#[xgroup] LocalComplexType),
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[xelement(name = "annotation", namespace = "http://www.w3.org/2001/XMLSchema")]
    struct Anno;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[xelement(name = "openContent", namespace = "http://www.w3.org/2001/XMLSchema")]
    struct OpenContent;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[xelement(name = "alternative", namespace = "http://www.w3.org/2001/XMLSchema")]
    struct Alt;

    #[derive(
        Debug, PartialEq, Serialize, Deserialize, SerializationGroup, DeserializationGroup,
    )]
    #[xelement(
        name = "element",
        namespace = "http://www.w3.org/2001/XMLSchema",
        children_order = "loose"
    )]
    #[xgroup(children_order = "strict")]
    struct TopLevelElement {
        #[xattribute(name = "name")]
        pub name: LocalName<'static>,
        // #[xvalue(default)]
        // pub annotation: Option<Anno>,
        #[xvalue(default)]
        pub child_1: Option<Child1>,
        #[xvalue(default)]
        pub alternative: Option<Alt>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[xelement(name = "element", namespace = "http://www.w3.org/2001/XMLSchema")]
    struct TopLevelElementWrapped(#[xgroup] pub TopLevelElement);

    const XHTML_THEAD: &str = r###"
    <xs:element xmlns="http://www.w3.org/1999/xhtml" xmlns:xs="http://www.w3.org/2001/XMLSchema" name="thead">
    <xs:complexType>
        <xs:sequence>
        <xs:element maxOccurs="unbounded" ref="tr"/>
        </xs:sequence>
        <xs:attributeGroup ref="attrs"/>
        <xs:attributeGroup ref="cellhalign"/>
        <xs:attributeGroup ref="cellvalign"/>
    </xs:complexType>
    </xs:element>
    "###;

    let a: TopLevelElement = xmlity_quick_xml::from_str(XHTML_THEAD.trim()).unwrap();
    println!("{:?}", a);

    let a_wrapped: TopLevelElementWrapped = xmlity_quick_xml::from_str(XHTML_THEAD.trim()).unwrap();
    println!("{:?}", a_wrapped);

    assert_eq!(a, a_wrapped.0);
}
