//! This uses parts of the XSD specification file to test the derive macros.
//!
//! This has nothing to do with the actual XSD crate/generator, it's just some tests that happen to use the same thing.
use pretty_assertions::assert_eq;

pub mod utils;
use utils::{clean_string, quick_xml_deserialize_test};

use std::fs;

use xmlity::types::{
    utils::{IgnoredAny, XmlRoot},
    value::{XmlComment, XmlDecl, XmlDoctype},
};
use xmlity::{Deserialize, Serialize, SerializeAttribute};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[xelement(name = "sequence", namespace = "http://www.w3.org/2001/XMLSchema")]
pub struct Sequence {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[xelement(name = "annotation", namespace = "http://www.w3.org/2001/XMLSchema")]
pub struct Annotation {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[xelement(name = "complexType", namespace = "http://www.w3.org/2001/XMLSchema")]
pub struct ComplexType {
    pub complex_content: ComplexContent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExtensionEntry {
    Sequence(Sequence),
    Attribute(Attribute),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[xelement(
    name = "complexContent",
    namespace = "http://www.w3.org/2001/XMLSchema"
)]
pub struct ComplexContent {
    content: Extension,
}

#[derive(Debug, Clone, SerializeAttribute, Deserialize, PartialEq)]
#[xattribute(name = "base")]
pub struct Base(String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[xelement(
    name = "extension",
    namespace = "http://www.w3.org/2001/XMLSchema",
    allow_unknown_children,
    allow_unknown_attributes
)]
pub struct Extension {
    #[xattribute]
    base: Base,
    entries: Vec<ExtensionEntry>,
}

#[derive(Debug, Clone, SerializeAttribute, Deserialize, PartialEq)]
#[xattribute(name = "name")]
pub struct Name(String);

#[derive(Debug, Clone, SerializeAttribute, Deserialize, PartialEq)]
#[xattribute(name = "id")]
pub struct Id(String);

#[derive(Debug, Clone, SerializeAttribute, Deserialize, PartialEq)]
#[xattribute(name = "type")]
pub struct Type(String);

#[derive(Debug, Clone, SerializeAttribute, Deserialize, PartialEq)]
#[xattribute(name = "use")]
pub struct Use(String);

#[derive(Debug, Clone, SerializeAttribute, Deserialize, PartialEq)]
#[xattribute(name = "default")]
pub struct Default(String);

#[derive(Debug, Clone, SerializeAttribute, Deserialize, PartialEq)]
#[xattribute(name = "ref")]
pub struct Ref(String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[xelement(
    name = "attribute",
    namespace = "http://www.w3.org/2001/XMLSchema",
    allow_unknown_children,
    allow_unknown_attributes
)]
pub struct Attribute {
    #[xattribute(default)]
    name: Option<Name>,
    #[xattribute(default)]
    attr_type: Option<Type>,
    #[xattribute(default)]
    attr_use: Option<Use>,
    #[xattribute(default)]
    attr_default: Option<Default>,
    #[xattribute(default)]
    attr_ref: Option<Ref>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[xelement(
    name = "element",
    namespace = "http://www.w3.org/2001/XMLSchema",
    allow_unknown_children,
    allow_unknown_attributes
)]
pub struct Element {
    #[xattribute]
    name: Name,
    #[xattribute]
    id: Id,
    annotation: Annotation,
    complex_type: ComplexType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SchemaEntry {
    Annotation(Annotation),
    Element(Element),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[xelement(name = "schema", namespace = "http://www.w3.org/2001/XMLSchema")]
pub struct Schema {
    #[xvalue(default)]
    pub sequence: Vec<SchemaEntry>,
}

const XSD_XML_DOCTYPE: &str = r###"xs:schema PUBLIC "-//W3C//DTD XMLSCHEMA 200102//EN" "XMLSchema.dtd" [

<!-- provide ID type information even for parsers which only read the
     internal subset -->
<!ATTLIST xs:schema          id  ID  #IMPLIED>
<!ATTLIST xs:complexType     id  ID  #IMPLIED>
<!ATTLIST xs:complexContent  id  ID  #IMPLIED>
<!ATTLIST xs:simpleContent   id  ID  #IMPLIED>
<!ATTLIST xs:extension       id  ID  #IMPLIED>
<!ATTLIST xs:element         id  ID  #IMPLIED>
<!ATTLIST xs:group           id  ID  #IMPLIED> 
<!ATTLIST xs:all             id  ID  #IMPLIED>
<!ATTLIST xs:choice          id  ID  #IMPLIED>
<!ATTLIST xs:sequence        id  ID  #IMPLIED>
<!ATTLIST xs:any             id  ID  #IMPLIED>
<!ATTLIST xs:anyAttribute    id  ID  #IMPLIED>
<!ATTLIST xs:attribute       id  ID  #IMPLIED>
<!ATTLIST xs:attributeGroup  id  ID  #IMPLIED>
<!ATTLIST xs:unique          id  ID  #IMPLIED>
<!ATTLIST xs:key             id  ID  #IMPLIED>
<!ATTLIST xs:keyref          id  ID  #IMPLIED>
<!ATTLIST xs:selector        id  ID  #IMPLIED>
<!ATTLIST xs:field           id  ID  #IMPLIED>
<!ATTLIST xs:include         id  ID  #IMPLIED>
<!ATTLIST xs:import          id  ID  #IMPLIED>
<!ATTLIST xs:redefine        id  ID  #IMPLIED>
<!ATTLIST xs:notation        id  ID  #IMPLIED>
<!--
     keep this schema XML1.0 DTD valid
  -->
        <!ENTITY % schemaAttrs 'xmlns:hfp CDATA #IMPLIED'>

        <!ELEMENT hfp:hasFacet EMPTY>
        <!ATTLIST hfp:hasFacet
                name NMTOKEN #REQUIRED>

        <!ELEMENT hfp:hasProperty EMPTY>
        <!ATTLIST hfp:hasProperty
                name NMTOKEN #REQUIRED
                value CDATA #REQUIRED>
<!--
        Make sure that processors that do not read the external
        subset will know about the various IDs we declare
  -->
        <!ATTLIST xs:simpleType id ID #IMPLIED>
        <!ATTLIST xs:maxExclusive id ID #IMPLIED>
        <!ATTLIST xs:minExclusive id ID #IMPLIED>
        <!ATTLIST xs:maxInclusive id ID #IMPLIED>
        <!ATTLIST xs:minInclusive id ID #IMPLIED>
        <!ATTLIST xs:totalDigits id ID #IMPLIED>
        <!ATTLIST xs:fractionDigits id ID #IMPLIED>
        <!ATTLIST xs:length id ID #IMPLIED>
        <!ATTLIST xs:minLength id ID #IMPLIED>
        <!ATTLIST xs:maxLength id ID #IMPLIED>
        <!ATTLIST xs:enumeration id ID #IMPLIED>
        <!ATTLIST xs:pattern id ID #IMPLIED>
        <!ATTLIST xs:appinfo id ID #IMPLIED>
        <!ATTLIST xs:documentation id ID #IMPLIED>
        <!ATTLIST xs:list id ID #IMPLIED>
        <!ATTLIST xs:union id ID #IMPLIED>
        ]"###;

fn xsd_struct() -> XmlRoot<Schema> {
    XmlRoot::new()
        .with_decl(XmlDecl::new("1.0", Some("UTF-8"), None))
        .with_comments([
            XmlComment::new(
                " XML Schema schema for XML Schemas: Part 1: Structures "
                    .as_bytes()
                    .to_owned(),
            ),
            XmlComment::new(
                " Note this schema is NOT the normative structures schema. "
                    .as_bytes()
                    .to_owned(),
            ),
            XmlComment::new(
                " The prose copy in the structures REC is the normative "
                    .as_bytes()
                    .to_owned(),
            ),
            XmlComment::new(
                " version (which shouldn't differ from this one except for "
                    .as_bytes()
                    .to_owned(),
            ),
            XmlComment::new(
                " this comment and entity expansions, but just in case "
                    .as_bytes()
                    .to_owned(),
            ),
        ])
        .with_doctype(XmlDoctype::new(XSD_XML_DOCTYPE.as_bytes()))
        .with_element(Schema {
            sequence: vec![
                SchemaEntry::Annotation(Annotation {}),
                SchemaEntry::Annotation(Annotation {}),
                SchemaEntry::Annotation(Annotation {}),
            ],
        })
}

#[test]
fn xsd_struct_deserialize() {
    let input_xml = fs::read_to_string("./tests/XMLSchema.xsd").unwrap();

    // Windows uses CRLF line endings, but the tests assume LF line endings. This is a hack to make the tests pass on Windows.
    #[cfg(windows)]
    let input_xml = input_xml.replace("\r\n", "\n");

    let actual: XmlRoot<Schema> = quick_xml_deserialize_test(&input_xml).unwrap();

    let expected = xsd_struct();

    assert_eq!(actual, expected);
}

const EMPTY_SCHEMA: &str = r####"
<?xml version='1.0' encoding='UTF-8'?>
<xs:schema targetNamespace="http://www.w3.org/2001/XMLSchema" blockDefault="#all" elementFormDefault="qualified" version="1.0" xmlns:xs="http://www.w3.org/2001/XMLSchema" xml:lang="EN" xmlns:hfp="http://www.w3.org/2001/XMLSchema-hasFacetAndProperty">
</xs:schema>
"####;

fn empty_schema() -> XmlRoot<Schema> {
    XmlRoot::new()
        .with_decl(XmlDecl::new("1.0", Some("UTF-8"), None))
        .with_element(Schema { sequence: vec![] })
}

#[test]
fn empty_schema_deserialize() {
    let actual: XmlRoot<Schema> = quick_xml_deserialize_test(&clean_string(EMPTY_SCHEMA)).unwrap();
    let expected = empty_schema();
    assert_eq!(actual, expected);
}

const SCHEMA_WITH_SINGLE_ANNOTATION: &str = r####"
<?xml version='1.0' encoding='UTF-8'?>
<xs:schema targetNamespace="http://www.w3.org/2001/XMLSchema" blockDefault="#all" elementFormDefault="qualified" version="1.0" xmlns:xs="http://www.w3.org/2001/XMLSchema" xml:lang="EN" xmlns:hfp="http://www.w3.org/2001/XMLSchema-hasFacetAndProperty">
    <xs:annotation>
        <xs:documentation xml:lang="EN">Schema for XML Schema.</xs:documentation>
    </xs:annotation>
</xs:schema>
"####;

fn schema_with_single_annotation() -> XmlRoot<Schema> {
    XmlRoot::new()
        .with_decl(XmlDecl::new("1.0", Some("UTF-8"), None))
        .with_element(Schema {
            sequence: vec![SchemaEntry::Annotation(Annotation {})],
        })
}

#[test]
fn schema_with_single_annotation_deserialize() {
    let actual: XmlRoot<Schema> =
        quick_xml_deserialize_test(SCHEMA_WITH_SINGLE_ANNOTATION).unwrap();
    let expected = schema_with_single_annotation();
    assert_eq!(actual, expected);
}

const SCHEMA_WITH_SINGLE_ANNOTATION_WITHOUT_DECL: &str = r####"
<xs:schema targetNamespace="http://www.w3.org/2001/XMLSchema" blockDefault="#all" elementFormDefault="qualified" version="1.0" xmlns:xs="http://www.w3.org/2001/XMLSchema" xml:lang="EN" xmlns:hfp="http://www.w3.org/2001/XMLSchema-hasFacetAndProperty">
    <xs:annotation>
        <xs:documentation xml:lang="EN">Schema for XML Schema.</xs:documentation>
    </xs:annotation>
</xs:schema>
"####;

fn schema_with_single_annotation_no_decl() -> Schema {
    Schema {
        sequence: vec![SchemaEntry::Annotation(Annotation {})],
    }
}

#[test]
fn schema_with_single_annotation_no_decl_deserialize() {
    let actual: Schema =
        quick_xml_deserialize_test(SCHEMA_WITH_SINGLE_ANNOTATION_WITHOUT_DECL).unwrap();
    let expected = schema_with_single_annotation_no_decl();
    assert_eq!(actual, expected);
}

#[test]
fn ignored_any_deserialize() {
    let actual: IgnoredAny =
        quick_xml_deserialize_test(SCHEMA_WITH_SINGLE_ANNOTATION_WITHOUT_DECL).unwrap();
    assert_eq!(actual, IgnoredAny);
}

const SINGLE_ATTRIBUTE: &str = r####"
<xs:attribute xmlns:xs="http://www.w3.org/2001/XMLSchema" name="targetNamespace" type="xs:anyURI"/>
"####;

fn single_attribute() -> Attribute {
    Attribute {
        name: Some(Name("targetNamespace".to_string())),
        attr_type: Some(Type("xs:anyURI".to_string())),
        attr_use: None,
        attr_default: None,
        attr_ref: None,
    }
}

#[test]
fn single_attribute_deserialize() {
    let actual: Attribute = quick_xml_deserialize_test(SINGLE_ATTRIBUTE).unwrap();
    let expected = single_attribute();
    assert_eq!(actual, expected);
}

const MULTIPLE_ATTRIBUTES: &str = r####"
<xs:attribute xmlns:xs="http://www.w3.org/2001/XMLSchema" name="targetNamespace" type="xs:anyURI"/>
<xs:attribute xmlns:xs="http://www.w3.org/2001/XMLSchema" name="version" type="xs:token"/>
<xs:attribute xmlns:xs="http://www.w3.org/2001/XMLSchema" name="finalDefault" type="xs:fullDerivationSet" use="optional" default=""/>
<xs:attribute xmlns:xs="http://www.w3.org/2001/XMLSchema" name="blockDefault" type="xs:blockSet" use="optional" default=""/>
<xs:attribute xmlns:xs="http://www.w3.org/2001/XMLSchema" name="attributeFormDefault" type="xs:formChoice" use="optional" default="unqualified"/>
<xs:attribute xmlns:xs="http://www.w3.org/2001/XMLSchema" name="elementFormDefault" type="xs:formChoice" use="optional" default="unqualified"/>
<xs:attribute xmlns:xs="http://www.w3.org/2001/XMLSchema" name="id" type="xs:ID"/>
<xs:attribute xmlns:xs="http://www.w3.org/2001/XMLSchema" ref="xml:lang"/>
"####;

fn multiple_attribute() -> Vec<Attribute> {
    vec![
        Attribute {
            name: Some(Name("targetNamespace".to_string())),
            attr_type: Some(Type("xs:anyURI".to_string())),
            attr_use: None,
            attr_default: None,
            attr_ref: None,
        },
        Attribute {
            name: Some(Name("version".to_string())),
            attr_type: Some(Type("xs:token".to_string())),
            attr_use: None,
            attr_default: None,
            attr_ref: None,
        },
        Attribute {
            name: Some(Name("finalDefault".to_string())),
            attr_type: Some(Type("xs:fullDerivationSet".to_string())),
            attr_use: Some(Use("optional".to_string())),
            attr_default: Some(Default("".to_string())),
            attr_ref: None,
        },
        Attribute {
            name: Some(Name("blockDefault".to_string())),
            attr_type: Some(Type("xs:blockSet".to_string())),
            attr_use: Some(Use("optional".to_string())),
            attr_default: Some(Default("".to_string())),
            attr_ref: None,
        },
        Attribute {
            name: Some(Name("attributeFormDefault".to_string())),
            attr_type: Some(Type("xs:formChoice".to_string())),
            attr_use: Some(Use("optional".to_string())),
            attr_default: Some(Default("unqualified".to_string())),
            attr_ref: None,
        },
        Attribute {
            name: Some(Name("elementFormDefault".to_string())),
            attr_type: Some(Type("xs:formChoice".to_string())),
            attr_use: Some(Use("optional".to_string())),
            attr_default: Some(Default("unqualified".to_string())),
            attr_ref: None,
        },
        Attribute {
            name: Some(Name("id".to_string())),
            attr_type: Some(Type("xs:ID".to_string())),
            attr_use: None,
            attr_default: None,
            attr_ref: None,
        },
        Attribute {
            name: None,
            attr_type: None,
            attr_use: None,
            attr_default: None,
            attr_ref: Some(Ref("xml:lang".to_string())),
        },
    ]
}

#[test]
fn multiple_attribute_deserialize() {
    let actual: Vec<Attribute> = quick_xml_deserialize_test(MULTIPLE_ATTRIBUTES).unwrap();
    let expected = multiple_attribute();
    assert_eq!(actual, expected);
}

#[test]
fn multiple_attribute_wrapped_deserialize() {
    let actual: Vec<ExtensionEntry> = quick_xml_deserialize_test(MULTIPLE_ATTRIBUTES).unwrap();
    let expected: Vec<ExtensionEntry> = multiple_attribute()
        .into_iter()
        .map(ExtensionEntry::Attribute)
        .collect();
    assert_eq!(actual, expected);
}

const SINGLE_CONTENT: &str = r####"
<xs:complexContent xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:extension base="xs:openAttrs">
    <xs:sequence>
        <xs:choice minOccurs="0" maxOccurs="unbounded">
            <xs:element ref="xs:include"/>
            <xs:element ref="xs:import"/>
            <xs:element ref="xs:redefine"/>
            <xs:element ref="xs:annotation"/>
        </xs:choice>
        <xs:sequence minOccurs="0" maxOccurs="unbounded">
        <xs:group ref="xs:schemaTop"/>
        <xs:element ref="xs:annotation" minOccurs="0" maxOccurs="unbounded"/>
        </xs:sequence>
    </xs:sequence>
    <xs:attribute name="targetNamespace" type="xs:anyURI"/>
    <xs:attribute name="version" type="xs:token"/>
    <xs:attribute name="finalDefault" type="xs:fullDerivationSet" use="optional" default=""/>
    <xs:attribute name="blockDefault" type="xs:blockSet" use="optional" default=""/>
    <xs:attribute name="attributeFormDefault" type="xs:formChoice" use="optional" default="unqualified"/>
    <xs:attribute name="elementFormDefault" type="xs:formChoice" use="optional" default="unqualified"/>
    <xs:attribute name="id" type="xs:ID"/>
    <xs:attribute ref="xml:lang"/>
    </xs:extension>
</xs:complexContent>
"####;

fn single_content() -> ComplexContent {
    ComplexContent {
        content: Extension {
            base: Base("xs:openAttrs".to_string()),
            entries: vec![
                ExtensionEntry::Sequence(Sequence {}),
                ExtensionEntry::Attribute(Attribute {
                    name: Some(Name("targetNamespace".to_string())),
                    attr_type: Some(Type("xs:anyURI".to_string())),
                    attr_use: None,
                    attr_default: None,
                    attr_ref: None,
                }),
                ExtensionEntry::Attribute(Attribute {
                    name: Some(Name("version".to_string())),
                    attr_type: Some(Type("xs:token".to_string())),
                    attr_use: None,
                    attr_default: None,
                    attr_ref: None,
                }),
                ExtensionEntry::Attribute(Attribute {
                    name: Some(Name("finalDefault".to_string())),
                    attr_type: Some(Type("xs:fullDerivationSet".to_string())),
                    attr_use: Some(Use("optional".to_string())),
                    attr_default: Some(Default("".to_string())),
                    attr_ref: None,
                }),
                ExtensionEntry::Attribute(Attribute {
                    name: Some(Name("blockDefault".to_string())),
                    attr_type: Some(Type("xs:blockSet".to_string())),
                    attr_use: Some(Use("optional".to_string())),
                    attr_default: Some(Default("".to_string())),
                    attr_ref: None,
                }),
                ExtensionEntry::Attribute(Attribute {
                    name: Some(Name("attributeFormDefault".to_string())),
                    attr_type: Some(Type("xs:formChoice".to_string())),
                    attr_use: Some(Use("optional".to_string())),
                    attr_default: Some(Default("unqualified".to_string())),
                    attr_ref: None,
                }),
                ExtensionEntry::Attribute(Attribute {
                    name: Some(Name("elementFormDefault".to_string())),
                    attr_type: Some(Type("xs:formChoice".to_string())),
                    attr_use: Some(Use("optional".to_string())),
                    attr_default: Some(Default("unqualified".to_string())),
                    attr_ref: None,
                }),
                ExtensionEntry::Attribute(Attribute {
                    name: Some(Name("id".to_string())),
                    attr_type: Some(Type("xs:ID".to_string())),
                    attr_use: None,
                    attr_default: None,
                    attr_ref: None,
                }),
                ExtensionEntry::Attribute(Attribute {
                    name: None,
                    attr_type: None,
                    attr_use: None,
                    attr_default: None,
                    attr_ref: Some(Ref("xml:lang".to_string())),
                }),
            ],
        },
    }
}

#[test]
fn single_content_deserialize() {
    let actual: ComplexContent = quick_xml_deserialize_test(SINGLE_CONTENT).unwrap();
    let expected = single_content();
    assert_eq!(actual, expected);
}

const SINGLE_ELEMENT: &str = r####"
<xs:element xmlns:xs="http://www.w3.org/2001/XMLSchema" name="schema" id="schema">
    <xs:annotation>
        <xs:documentation source="http://www.w3.org/TR/xmlschema-1/#element-schema"/>
    </xs:annotation>
    <xs:complexType>
        <xs:complexContent>
            <xs:extension base="xs:openAttrs">
            <xs:sequence>
                <xs:choice minOccurs="0" maxOccurs="unbounded">
                    <xs:element ref="xs:include"/>
                    <xs:element ref="xs:import"/>
                    <xs:element ref="xs:redefine"/>
                    <xs:element ref="xs:annotation"/>
                </xs:choice>
                <xs:sequence minOccurs="0" maxOccurs="unbounded">
                <xs:group ref="xs:schemaTop"/>
                <xs:element ref="xs:annotation" minOccurs="0" maxOccurs="unbounded"/>
                </xs:sequence>
            </xs:sequence>
            <xs:attribute name="targetNamespace" type="xs:anyURI"/>
            <xs:attribute name="version" type="xs:token"/>
            <xs:attribute name="finalDefault" type="xs:fullDerivationSet" use="optional" default=""/>
            <xs:attribute name="blockDefault" type="xs:blockSet" use="optional" default=""/>
            <xs:attribute name="attributeFormDefault" type="xs:formChoice" use="optional" default="unqualified"/>
            <xs:attribute name="elementFormDefault" type="xs:formChoice" use="optional" default="unqualified"/>
            <xs:attribute name="id" type="xs:ID"/>
            <xs:attribute ref="xml:lang"/>
            </xs:extension>
        </xs:complexContent>
    </xs:complexType>

    <xs:key name="element">
        <xs:selector xpath="xs:element"/>
        <xs:field xpath="@name"/>
    </xs:key>

    <xs:key name="attribute">
        <xs:selector xpath="xs:attribute"/>
        <xs:field xpath="@name"/>
    </xs:key>

    <xs:key name="type">
        <xs:selector xpath="xs:complexType|xs:simpleType"/>
        <xs:field xpath="@name"/>
    </xs:key>
    
    <xs:key name="group">
        <xs:selector xpath="xs:group"/>
        <xs:field xpath="@name"/>
    </xs:key>
    
    <xs:key name="attributeGroup">
        <xs:selector xpath="xs:attributeGroup"/>
        <xs:field xpath="@name"/>
    </xs:key>
    
    <xs:key name="notation">
        <xs:selector xpath="xs:notation"/>
        <xs:field xpath="@name"/>
    </xs:key>

    <xs:key name="identityConstraint">
        <xs:selector xpath=".//xs:key|.//xs:unique|.//xs:keyref"/>
        <xs:field xpath="@name"/>
    </xs:key>
</xs:element>
"####;

fn single_element() -> Element {
    Element {
        name: Name("schema".to_string()),
        id: Id("schema".to_string()),
        annotation: Annotation {},
        complex_type: ComplexType {
            complex_content: ComplexContent {
                content: Extension {
                    base: Base("xs:openAttrs".to_string()),
                    entries: vec![
                        ExtensionEntry::Sequence(Sequence {}),
                        ExtensionEntry::Attribute(Attribute {
                            name: Some(Name("targetNamespace".to_string())),
                            attr_type: Some(Type("xs:anyURI".to_string())),
                            attr_use: None,
                            attr_default: None,
                            attr_ref: None,
                        }),
                        ExtensionEntry::Attribute(Attribute {
                            name: Some(Name("version".to_string())),
                            attr_type: Some(Type("xs:token".to_string())),
                            attr_use: None,
                            attr_default: None,
                            attr_ref: None,
                        }),
                        ExtensionEntry::Attribute(Attribute {
                            name: Some(Name("finalDefault".to_string())),
                            attr_type: Some(Type("xs:fullDerivationSet".to_string())),
                            attr_use: Some(Use("optional".to_string())),
                            attr_default: Some(Default("".to_string())),
                            attr_ref: None,
                        }),
                        ExtensionEntry::Attribute(Attribute {
                            name: Some(Name("blockDefault".to_string())),
                            attr_type: Some(Type("xs:blockSet".to_string())),
                            attr_use: Some(Use("optional".to_string())),
                            attr_default: Some(Default("".to_string())),
                            attr_ref: None,
                        }),
                        ExtensionEntry::Attribute(Attribute {
                            name: Some(Name("attributeFormDefault".to_string())),
                            attr_type: Some(Type("xs:formChoice".to_string())),
                            attr_use: Some(Use("optional".to_string())),
                            attr_default: Some(Default("unqualified".to_string())),
                            attr_ref: None,
                        }),
                        ExtensionEntry::Attribute(Attribute {
                            name: Some(Name("elementFormDefault".to_string())),
                            attr_type: Some(Type("xs:formChoice".to_string())),
                            attr_use: Some(Use("optional".to_string())),
                            attr_default: Some(Default("unqualified".to_string())),
                            attr_ref: None,
                        }),
                        ExtensionEntry::Attribute(Attribute {
                            name: Some(Name("id".to_string())),
                            attr_type: Some(Type("xs:ID".to_string())),
                            attr_use: None,
                            attr_default: None,
                            attr_ref: None,
                        }),
                        ExtensionEntry::Attribute(Attribute {
                            name: None,
                            attr_type: None,
                            attr_use: None,
                            attr_default: None,
                            attr_ref: Some(Ref("xml:lang".to_string())),
                        }),
                    ],
                },
            },
        },
    }
}

#[test]
fn single_element_deserialize() {
    let actual: Element = quick_xml_deserialize_test(SINGLE_ELEMENT).unwrap();
    let expected = single_element();
    assert_eq!(actual, expected);
}

const SCHEMA_WITH_SINGLE_ELEMENT: &str = r####"
<?xml version='1.0' encoding='UTF-8'?>
<xs:schema targetNamespace="http://www.w3.org/2001/XMLSchema" blockDefault="#all" elementFormDefault="qualified" version="1.0" xmlns:xs="http://www.w3.org/2001/XMLSchema" xml:lang="EN" xmlns:hfp="http://www.w3.org/2001/XMLSchema-hasFacetAndProperty">
    <xs:element name="schema" id="schema">
        <xs:annotation>
            <xs:documentation source="http://www.w3.org/TR/xmlschema-1/#element-schema"/>
        </xs:annotation>
        <xs:complexType>
            <xs:complexContent>
                <xs:extension base="xs:openAttrs">
                <xs:sequence>
                    <xs:choice minOccurs="0" maxOccurs="unbounded">
                        <xs:element ref="xs:include"/>
                        <xs:element ref="xs:import"/>
                        <xs:element ref="xs:redefine"/>
                        <xs:element ref="xs:annotation"/>
                    </xs:choice>
                    <xs:sequence minOccurs="0" maxOccurs="unbounded">
                    <xs:group ref="xs:schemaTop"/>
                    <xs:element ref="xs:annotation" minOccurs="0" maxOccurs="unbounded"/>
                    </xs:sequence>
                </xs:sequence>
                <xs:attribute name="targetNamespace" type="xs:anyURI"/>
                <xs:attribute name="version" type="xs:token"/>
                <xs:attribute name="finalDefault" type="xs:fullDerivationSet" use="optional" default=""/>
                <xs:attribute name="blockDefault" type="xs:blockSet" use="optional" default=""/>
                <xs:attribute name="attributeFormDefault" type="xs:formChoice" use="optional" default="unqualified"/>
                <xs:attribute name="elementFormDefault" type="xs:formChoice" use="optional" default="unqualified"/>
                <xs:attribute name="id" type="xs:ID"/>
                <xs:attribute ref="xml:lang"/>
                </xs:extension>
            </xs:complexContent>
        </xs:complexType>

        <xs:key name="element">
            <xs:selector xpath="xs:element"/>
            <xs:field xpath="@name"/>
        </xs:key>

        <xs:key name="attribute">
            <xs:selector xpath="xs:attribute"/>
            <xs:field xpath="@name"/>
        </xs:key>

        <xs:key name="type">
            <xs:selector xpath="xs:complexType|xs:simpleType"/>
            <xs:field xpath="@name"/>
        </xs:key>
        
        <xs:key name="group">
            <xs:selector xpath="xs:group"/>
            <xs:field xpath="@name"/>
        </xs:key>
        
        <xs:key name="attributeGroup">
            <xs:selector xpath="xs:attributeGroup"/>
            <xs:field xpath="@name"/>
        </xs:key>
        
        <xs:key name="notation">
            <xs:selector xpath="xs:notation"/>
            <xs:field xpath="@name"/>
        </xs:key>

        <xs:key name="identityConstraint">
            <xs:selector xpath=".//xs:key|.//xs:unique|.//xs:keyref"/>
            <xs:field xpath="@name"/>
        </xs:key>
    </xs:element>
</xs:schema>
"####;

fn schema_with_single_element() -> XmlRoot<Schema> {
    XmlRoot::new()
        .with_decl(XmlDecl::new("1.0", Some("UTF-8"), None))
        .with_element(Schema {
            sequence: vec![SchemaEntry::Element(single_element())],
        })
}

#[test]
fn schema_with_single_element_deserialize() {
    let actual: XmlRoot<Schema> = quick_xml_deserialize_test(SCHEMA_WITH_SINGLE_ELEMENT).unwrap();
    let expected = schema_with_single_element();
    assert_eq!(actual, expected);
}
