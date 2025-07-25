pub mod linkbase_ref_items {
    #[derive(
        Debug, ::xmlity::SerializationGroup, ::xmlity::DeserializationGroup, PartialEq, Clone,
    )]
    pub struct LinkbaseRef {}
}
#[derive(Debug, ::xmlity::Serialize, ::xmlity::Deserialize, PartialEq, Clone)]
pub enum LinkbaseRef {
    #[xelement(
        name = "linkbaseRef",
        namespace = "http://www.xbrl.org/2003/linkbase",
        allow_unknown_attributes = "any"
    )]
    LinkbaseRef(#[xgroup] linkbase_ref_items::LinkbaseRef),
}

const LINKBASE_REF: &str = r###"
<link:linkbaseRef 
    xmlns:link="http://www.xbrl.org/2003/linkbase" />
"###;

#[test]
fn linkbase_ref() {
    let direct: LinkbaseRef =
        xmlity_quick_xml::from_str(LINKBASE_REF.trim()).expect("Failed to parse linkbaseRef XML");

    let element: xmlity::value::XmlValue =
        xmlity_quick_xml::from_str(LINKBASE_REF.trim()).expect("Failed to parse linkbaseRef XML");

    let indirect: LinkbaseRef =
        xmlity::Deserialize::deserialize(&element).expect("Failed to deserialize linkbaseRef XML");

    assert_eq!(direct, indirect);
}
