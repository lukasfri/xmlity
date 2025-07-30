//! Ordering and Unknown Content Examples
//!
//! Demonstrates ordering constraints and unknown content handling:
//! - Attribute and element ordering options
//! - Unknown attributes and children handling
//! - Whitespace and comment handling

use xmlity::{Deserialize, Serialize};
use xmlity_quick_xml::{from_str, to_string};

// Strict ordering requirements
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(
    name = "strict-order",
    attribute_order = "strict",
    children_order = "strict"
)]
struct StrictOrder {
    #[xattribute(name = "first-attr")]
    first_attr: String,

    #[xattribute(name = "second-attr")]
    second_attr: String,

    #[xelement(name = "first-child")]
    first_child: String,

    #[xelement(name = "second-child")]
    second_child: String,
}

// Loose ordering (elements can be in any order and can have unknowns)
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(
    name = "loose-order",
    attribute_order = "none",
    children_order = "none",
    allow_unknown_attributes = "any",
    allow_unknown_children = "any"
)]
struct LooseOrder {
    #[xattribute(name = "important-attr")]
    important_attr: String,

    #[xelement(name = "required-child")]
    required_child: String,

    #[xelement(name = "another-child")]
    another_child: String,
}

// Allow unknown content
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(
    name = "flexible",
    allow_unknown_attributes = "any",
    allow_unknown_children = "any"
)]
struct FlexibleElement {
    #[xattribute(name = "known-attr")]
    known_attr: String,

    #[xelement(name = "known-child")]
    known_child: String,
}

// Allow unknown only at end
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(
    name = "semi-flexible",
    allow_unknown_attributes = "at_end",
    allow_unknown_children = "at_end"
)]
struct SemiFlexibleElement {
    #[xattribute(name = "first-attr")]
    first_attr: String,

    #[xattribute(name = "second-attr")]
    second_attr: String,

    #[xelement(name = "first-child")]
    first_child: String,

    #[xelement(name = "second-child")]
    second_child: String,
}

// Deserialize any name
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(deserialize_any_name = true)]
struct AnyNameElement {
    #[xattribute(name = "id")]
    id: String,

    #[xvalue]
    content: String,
}

// Ignore whitespace and comments
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(
    name = "document",
    ignore_whitespace = "none",
    ignore_comments = "none"
)]
struct DocumentWithWhitespace {
    #[xelement(name = "title")]
    title: String,

    #[xvalue(extendable = true)]
    content: String,
}

impl Extend<DocumentWithWhitespace> for DocumentWithWhitespace {
    fn extend<T: IntoIterator<Item = DocumentWithWhitespace>>(&mut self, iter: T) {
        for doc in iter {
            self.content.push_str(&doc.content);
        }
    }
}

// Complex example combining multiple options
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(
    name = "advanced-config",
    attribute_order = "none",
    children_order = "none",
    allow_unknown_attributes = "at_end",
    allow_unknown_children = "at_end",
    ignore_whitespace = "any",
    ignore_comments = "any"
)]
struct AdvancedConfig {
    #[xattribute(name = "version")]
    version: String,

    #[xattribute(name = "env")]
    environment: String,

    #[xelement(name = "database")]
    database: DatabaseSettings,

    #[xelement(name = "logging")]
    logging: LoggingSettings,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "database")]
struct DatabaseSettings {
    #[xattribute(name = "host")]
    host: String,

    #[xattribute(name = "port", default)]
    port: u16,

    #[xelement(name = "name")]
    name: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "logging")]
struct LoggingSettings {
    #[xattribute(name = "level")]
    level: String,

    #[xelement(name = "output")]
    output: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Ordering and Unknown Content ===\n");

    // Strict ordering - must be exact
    let strict = StrictOrder {
        first_attr: "value1".to_string(),
        second_attr: "value2".to_string(),
        first_child: "child1".to_string(),
        second_child: "child2".to_string(),
    };

    let xml = to_string(&strict)?;
    println!("Strict Order:\n{}\n", xml);

    let parsed: StrictOrder = from_str(&xml)?;
    assert_eq!(strict, parsed);

    // Loose ordering - allows unknowns between required elements
    let loose = LooseOrder {
        important_attr: "important".to_string(),
        required_child: "required".to_string(),
        another_child: "another".to_string(),
    };

    let xml = to_string(&loose)?;
    println!("Loose Order:\n{}\n", xml);

    // Test with additional unknown attributes and children
    let xml_with_unknowns = r#"<loose-order important-attr="important" unknown-attr="ignored" extra-attr="also-ignored">
        <unknown-element>ignored</unknown-element>
        <required-child>required</required-child>
        <mystery-child>mystery</mystery-child>
        <another-child>another</another-child>
        <final-unknown>also ignored</final-unknown>
    </loose-order>"#;

    let parsed: LooseOrder = from_str(xml_with_unknowns)?;
    assert_eq!(loose, parsed);
    println!("Parsed with unknown content successfully\n");

    // Flexible element - allows any unknown content
    let flexible = FlexibleElement {
        known_attr: "known".to_string(),
        known_child: "child".to_string(),
    };

    let xml = to_string(&flexible)?;
    println!("Flexible Element:\n{}\n", xml);

    let xml_with_lots_of_unknowns = r#"<flexible known-attr="known" mystery1="value1" mystery2="value2">
        <unknown1>content1</unknown1>
        <known-child>child</known-child>
        <unknown2>content2</unknown2>
        <unknown3>content3</unknown3>
    </flexible>"#;

    let parsed: FlexibleElement = from_str(xml_with_lots_of_unknowns)?;
    assert_eq!(flexible, parsed);

    // Semi-flexible - unknowns only at end
    let semi_flexible = SemiFlexibleElement {
        first_attr: "first".to_string(),
        second_attr: "second".to_string(),
        first_child: "child1".to_string(),
        second_child: "child2".to_string(),
    };

    let xml = to_string(&semi_flexible)?;
    println!("Semi-Flexible Element:\n{}\n", xml);

    let xml_with_end_unknowns = r#"<semi-flexible first-attr="first" second-attr="second" unknown-attr="at-end">
        <first-child>child1</first-child>
        <second-child>child2</second-child>
        <unknown-child>at end</unknown-child>
    </semi-flexible>"#;

    let parsed: SemiFlexibleElement = from_str(xml_with_end_unknowns)?;
    assert_eq!(semi_flexible, parsed);

    // Any name element
    let any_name = AnyNameElement {
        id: "123".to_string(),
        content: "This can be any element name".to_string(),
    };

    // Test with different element names
    let different_names = [
        r#"<random-name id="123">This can be any element name</random-name>"#,
        r#"<another-name id="123">This can be any element name</another-name>"#,
        r#"<completely-different id="123">This can be any element name</completely-different>"#,
    ];

    for xml_variant in &different_names {
        let parsed: AnyNameElement = from_str(xml_variant)?;
        assert_eq!(any_name, parsed);
    }
    println!("Any-name element parsed with different names\n");

    // Document with whitespace handling
    let doc_with_whitespace = DocumentWithWhitespace {
        title: "Important Document".to_string(),
        content: "This is the main content.".to_string(),
    };

    let xml = to_string(&doc_with_whitespace)?;
    println!("Document with Whitespace:\n{}\n", xml);

    let parsed: DocumentWithWhitespace = from_str(&xml)?;
    assert_eq!(doc_with_whitespace, parsed);

    // Advanced config combining multiple options
    let advanced_config = AdvancedConfig {
        version: "1.0".to_string(),
        environment: "production".to_string(),
        database: DatabaseSettings {
            host: "db.example.com".to_string(),
            port: 5432,
            name: "app_db".to_string(),
        },
        logging: LoggingSettings {
            level: "info".to_string(),
            output: "stdout".to_string(),
        },
    };

    let xml = to_string(&advanced_config)?;
    println!("Advanced Config:\n{}\n", xml);

    let parsed: AdvancedConfig = from_str(&xml)?;
    assert_eq!(advanced_config, parsed);

    println!("All examples passed!");
    Ok(())
}
