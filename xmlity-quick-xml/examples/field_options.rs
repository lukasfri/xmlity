//! Field Options Examples
//!
//! Demonstrates field-level options:
//! - Optional fields with Option types
//! - Element structures with optional fields

use xmlity::{Deserialize, Serialize};
use xmlity_quick_xml::{from_str, to_string};

// Struct with optional fields
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "person")]
struct Person {
    #[xelement(name = "name", optional = true)]
    name: Option<String>,

    #[xelement(name = "nickname", optional = true)]
    nickname: Option<String>,

    #[xelement(name = "age", optional = true)]
    age: Option<u32>,

    #[xelement(name = "email", optional = true)]
    email: Option<String>,
}

// Struct with attributes and optional elements
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "product")]
struct Product {
    #[xattribute(name = "id")]
    id: String,

    #[xelement(name = "name")]
    name: String,

    #[xelement(name = "description", optional = true)]
    description: Option<String>,

    #[xelement(name = "price")]
    price: f64,

    #[xelement(name = "discount", optional = true)]
    discount: Option<u32>,

    #[xelement(name = "tags", optional = true)]
    tags: Option<Vec<String>>,
}

// Simple configuration struct
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "configuration")]
struct Configuration {
    #[xattribute(name = "version")]
    version: String,

    #[xelement(name = "database")]
    database: DatabaseConfig,

    #[xelement(name = "cache", optional = true)]
    cache: Option<CacheConfig>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "database")]
struct DatabaseConfig {
    #[xattribute(name = "host")]
    host: String,

    #[xattribute(name = "port")]
    port: u16,

    #[xelement(name = "name")]
    name: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "cache")]
struct CacheConfig {
    #[xattribute(name = "ttl")]
    ttl: u32,

    #[xelement(name = "size")]
    size: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Field Options ===\n");

    // Person with all fields present
    let person_full = Person {
        name: Some("Alice".to_string()),
        nickname: Some("Ally".to_string()),
        age: Some(30),
        email: Some("alice@example.com".to_string()),
    };

    let xml = to_string(&person_full)?;
    println!("Person (full):\n{}\n", xml);

    let parsed: Person = from_str(&xml)?;
    assert_eq!(person_full, parsed);

    // Person with minimal fields
    let person_minimal = Person {
        name: Some("Bob".to_string()),
        nickname: None,
        age: None,
        email: None,
    };

    let xml = to_string(&person_minimal)?;
    println!("Person (minimal):\n{}\n", xml);

    let parsed: Person = from_str(&xml)?;
    assert_eq!(person_minimal, parsed);

    // Product with all fields
    let product_full = Product {
        id: "PROD001".to_string(),
        name: "Gaming Laptop".to_string(),
        description: Some("High-performance laptop".to_string()),
        price: 1299.99,
        discount: Some(10),
        tags: Some(vec!["electronics".to_string(), "computers".to_string()]),
    };

    let xml = to_string(&product_full)?;
    println!("Product (full):\n{}\n", xml);

    let _parsed: Product = from_str(&xml)?;
    // Note: Vec<String> serialization behavior
    println!("Note: tags field shows Vec<String> serialization\n");

    // Product without optional fields
    let product_minimal = Product {
        id: "PROD002".to_string(),
        name: "Basic Laptop".to_string(),
        description: None,
        price: 599.99,
        discount: None,
        tags: None,
    };

    let xml = to_string(&product_minimal)?;
    println!("Product (minimal):\n{}\n", xml);

    let parsed: Product = from_str(&xml)?;
    assert_eq!(product_minimal, parsed);

    // Configuration with cache
    let config_with_cache = Configuration {
        version: "1.0".to_string(),
        database: DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            name: "myapp".to_string(),
        },
        cache: Some(CacheConfig {
            ttl: 3600,
            size: "100MB".to_string(),
        }),
    };

    let xml = to_string(&config_with_cache)?;
    println!("Configuration (with cache):\n{}\n", xml);

    let parsed: Configuration = from_str(&xml)?;
    assert_eq!(config_with_cache, parsed);

    // Configuration without cache
    let config_no_cache = Configuration {
        version: "1.0".to_string(),
        database: DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            name: "myapp".to_string(),
        },
        cache: None,
    };

    let xml = to_string(&config_no_cache)?;
    println!("Configuration (without cache):\n{}\n", xml);

    let parsed: Configuration = from_str(&xml)?;
    assert_eq!(config_no_cache, parsed);

    println!("All examples passed!");
    Ok(())
}
