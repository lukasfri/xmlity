use xmlity::{DeserializationGroup, Deserialize, SerializationGroup, Serialize, XmlNamespace};

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
