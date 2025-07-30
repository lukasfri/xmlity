//! Attributes Examples
//!
//! Demonstrates XML attributes with xmlity:
//! - Standalone attributes with SerializeAttribute
//! - Inline attributes with #[xattribute]
//! - Optional and namespaced attributes

use xmlity::{Deserialize, Serialize, SerializeAttribute};
use xmlity_quick_xml::{from_str, to_string};

// Standalone attribute type
#[derive(Debug, PartialEq, SerializeAttribute, Deserialize)]
#[xattribute(name = "id")]
struct Id(String);

#[derive(Debug, PartialEq, SerializeAttribute, Deserialize)]
#[xattribute(name = "version")]
struct Version(String);

// Element with deferred attributes
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "document")]
struct Document {
    #[xattribute(deferred = true)]
    id: Id,
    #[xattribute(deferred = true)]
    version: Version,
    #[xelement(name = "title")]
    title: String,
    #[xvalue]
    content: String,
}

// Element with inline attributes
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "product")]
struct Product {
    #[xattribute(name = "sku")]
    sku: String,
    #[xattribute(name = "category")]
    category: String,
    #[xelement(name = "name")]
    name: String,
    #[xelement(name = "price")]
    price: f64,
}

// Element with optional attributes
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "item")]
struct Item {
    #[xattribute(name = "id")]
    id: String,
    #[xattribute(name = "optional-attr", optional = true)]
    optional_attr: Option<String>,
    #[xvalue]
    value: String,
}

// Element with namespaced attributes
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "config", namespace = "http://example.com/config")]
struct Config {
    #[xattribute(name = "env", namespace = "http://example.com/meta")]
    environment: String,
    #[xattribute(name = "debug", preferred_prefix = "meta", enforce_prefix = true)]
    debug_mode: bool,
    #[xelement(name = "setting")]
    setting: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Attributes ===\n");

    // Document with deferred attributes
    let document = Document {
        id: Id("doc-123".to_string()),
        version: Version("1.0".to_string()),
        title: "Sample Document".to_string(),
        content: "This is the document content.".to_string(),
    };

    let xml = to_string(&document)?;
    println!("Document:\n{}\n", xml);

    let parsed: Document = from_str(&xml)?;
    assert_eq!(document, parsed);

    // Product with inline attributes
    let product = Product {
        sku: "RUST-BOOK-001".to_string(),
        category: "books".to_string(),
        name: "Learning Rust".to_string(),
        price: 29.99,
    };

    let xml = to_string(&product)?;
    println!("Product:\n{}\n", xml);

    let parsed: Product = from_str(&xml)?;
    assert_eq!(product, parsed);

    // Item with optional attribute (present)
    let item_with_attr = Item {
        id: "item-1".to_string(),
        optional_attr: Some("optional-value".to_string()),
        value: "Item content".to_string(),
    };

    let xml = to_string(&item_with_attr)?;
    println!("Item with optional attribute:\n{}\n", xml);

    let parsed: Item = from_str(&xml)?;
    assert_eq!(item_with_attr, parsed);

    // Item with optional attribute (absent)
    let item_without_attr = Item {
        id: "item-2".to_string(),
        optional_attr: None,
        value: "Another item".to_string(),
    };

    let xml = to_string(&item_without_attr)?;
    println!("Item without optional attribute:\n{}\n", xml);

    let parsed: Item = from_str(&xml)?;
    assert_eq!(item_without_attr, parsed);

    // Config with namespaced attributes
    let config = Config {
        environment: "production".to_string(),
        debug_mode: false,
        setting: "some-value".to_string(),
    };

    let xml = to_string(&config)?;
    println!("Config:\n{}\n", xml);

    let parsed: Config = from_str(&xml)?;
    assert_eq!(config, parsed);

    println!("All examples passed!");
    Ok(())
}
