use xmlity::{Deserialize, Serialize, XmlNamespace};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[xelement(
    name = "a",
    namespace_expr = XmlNamespace::XS
)]
pub struct A {
    #[xattribute(name = "b")]
    pub b: String,
    #[xattribute(name = "c", optional)]
    pub c: Option<String>,
    #[xattribute(name = "d", optional)]
    pub d: Option<String>,
    #[xattribute(name = "e", namespace_expr = XmlNamespace::XML, optional)]
    pub e: Option<String>,
}

const A_CASE1: &str = r###"
<xs:a xmlns:xs="http://www.w3.org/2001/XMLSchema"
           d="D-Value" xml:e="E_VALUE"
           b="b_value"
           c="CValue"/>
"###;

const A_CASE2: &str = r###"
<xs:a c="CValue" xml:e="E_VALUE"
    xmlns:xs="http://www.w3.org/2001/XMLSchema"
    b="b_value"
    xmlns="http://www.w3.org/1999/xhtml"
    xmlns:xml="http://www.w3.org/XML/1998/namespace"
    d="D-Value"/>
"###;

const A_CASE3: &str = r###"
<xs:a 
    xmlns:xs="http://www.w3.org/2001/XMLSchema"
    xmlns="http://www.w3.org/1999/xhtml"
    xmlns:xml="http://www.w3.org/XML/1998/namespace"
    c="CValue" xml:e="E_VALUE"
    b="b_value"
    d="D-Value"/>
"###;

#[rstest::rstest]
#[case(A_CASE1)]
#[case(A_CASE2)]
#[case(A_CASE3)]
fn attribute_orders_a(#[case] xml: &str) {
    let a: A = xmlity_quick_xml::from_str(xml.trim()).unwrap();

    assert_eq!(a.b, "b_value");
    assert_eq!(a.c, Some("CValue".to_string()));
    assert_eq!(a.d, Some("D-Value".to_string()));
    assert_eq!(a.e, Some("E_VALUE".to_string()));
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[xelement(
    name = "F",
    namespace_expr = XmlNamespace::XS
)]
pub struct F {
    #[xattribute(name = "b")]
    pub b: String,
}

const F_CASE1: &str = r###"
<xs:F xmlns:xs="http://www.w3.org/2001/XMLSchema"
           b="b_value"/>
"###;

const F_CASE2: &str = r###"
<xs:F
    xmlns:xs="http://www.w3.org/2001/XMLSchema"
    b="b_value"
    xmlns="http://www.w3.org/1999/xhtml"/>
"###;

const F_CASE3: &str = r###"
<xs:F 
    xmlns:xs="http://www.w3.org/2001/XMLSchema"
    xmlns="http://www.w3.org/1999/xhtml"
    b="b_value"/>
"###;

#[rstest::rstest]
#[case(F_CASE1)]
#[case(F_CASE2)]
#[case(F_CASE3)]
fn attribute_orders_f(#[case] xml: &str) {
    let a: F = xmlity_quick_xml::from_str(xml.trim()).unwrap();

    assert_eq!(a.b, "b_value");
}
