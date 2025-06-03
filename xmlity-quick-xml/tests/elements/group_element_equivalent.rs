use xmlity::{DeserializationGroup, Deserialize, SerializationGroup, Serialize, XmlNamespace};

#[test]
fn wrapped_element_equals_element() {
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

    assert_eq!(a, a_wrapped.0);
}
