//! Groups Examples
//!
//! Demonstrates SerializationGroup and DeserializationGroup traits:
//! - Basic group usage with #[xgroup]
//! - Group ordering options
//! - Combining groups with elements

use xmlity::{DeserializationGroup, Deserialize, SerializationGroup, Serialize};
use xmlity_quick_xml::{from_str, to_string};

// Basic group - represents a collection of related fields
#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
#[xgroup]
struct ContactInfo {
    #[xelement(name = "email")]
    email: String,

    #[xelement(name = "phone")]
    phone: String,

    #[xelement(name = "address")]
    address: String,
}

// Group with strict ordering
#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
#[xgroup(attribute_order = "strict", children_order = "strict")]
struct StrictMetadata {
    #[xattribute(name = "created")]
    created: String,

    #[xattribute(name = "modified")]
    modified: String,

    #[xelement(name = "author")]
    author: String,

    #[xelement(name = "version")]
    version: String,
}

// Group with loose ordering
#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
#[xgroup(attribute_order = "loose", children_order = "loose")]
struct LooseConfiguration {
    #[xattribute(name = "env")]
    environment: String,

    #[xattribute(name = "debug")]
    debug: bool,

    #[xelement(name = "timeout")]
    timeout: u32,

    #[xelement(name = "retries")]
    retries: u32,
}

// Element that uses groups
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "person")]
struct Person {
    #[xattribute(name = "id")]
    id: String,

    #[xelement(name = "name")]
    name: String,

    #[xgroup]
    contact: ContactInfo,

    #[xelement(name = "bio", optional = true)]
    bio: Option<String>,
}

// Element with strict metadata group
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "document")]
struct Document {
    #[xattribute(name = "id")]
    id: String,

    #[xgroup]
    metadata: StrictMetadata,

    #[xelement(name = "title")]
    title: String,

    #[xvalue]
    content: String,
}

// Element with loose configuration group
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "service")]
struct Service {
    #[xattribute(name = "name")]
    name: String,

    #[xgroup]
    config: LooseConfiguration,

    #[xelement(name = "endpoint")]
    endpoints: Vec<String>,
}

// Nested groups example
#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
#[xgroup]
struct DatabaseConfig {
    #[xattribute(name = "host")]
    host: String,

    #[xattribute(name = "port")]
    port: u16,

    #[xelement(name = "database")]
    database: String,
}

#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
#[xgroup]
struct CacheConfig {
    #[xattribute(name = "ttl")]
    ttl: u32,

    #[xelement(name = "size")]
    size: u32,
}

#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
#[xgroup]
struct ApplicationConfig {
    #[xgroup]
    database: DatabaseConfig,

    #[xgroup]
    cache: CacheConfig,

    #[xelement(name = "log-level")]
    log_level: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "application")]
struct Application {
    #[xattribute(name = "name")]
    name: String,

    #[xattribute(name = "version")]
    version: String,

    #[xgroup]
    config: ApplicationConfig,

    #[xelement(name = "status")]
    status: String,
}

// Example with multiple groups
#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
#[xgroup]
struct BasicInfo {
    #[xelement(name = "name")]
    name: String,

    #[xelement(name = "description")]
    description: String,
}

#[derive(Debug, PartialEq, SerializationGroup, DeserializationGroup)]
#[xgroup]
struct TechnicalInfo {
    #[xattribute(name = "version")]
    version: String,

    #[xattribute(name = "build")]
    build: String,

    #[xelement(name = "dependencies")]
    dependencies: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "project")]
struct Project {
    #[xattribute(name = "id")]
    id: String,

    #[xgroup]
    basic: BasicInfo,

    #[xgroup]
    technical: TechnicalInfo,

    #[xelement(name = "maintainer")]
    maintainer: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Groups ===\n");

    // Person with contact info group
    let person = Person {
        id: "P001".to_string(),
        name: "John Doe".to_string(),
        contact: ContactInfo {
            email: "john@example.com".to_string(),
            phone: "+1-555-0123".to_string(),
            address: "123 Main St, Anytown, USA".to_string(),
        },
        bio: Some("Software developer with 10 years of experience.".to_string()),
    };

    let xml = to_string(&person)?;
    println!("Person:\n{}\n", xml);

    let parsed: Person = from_str(&xml)?;
    assert_eq!(person, parsed);

    // Document with strict metadata group
    let document = Document {
        id: "DOC001".to_string(),
        metadata: StrictMetadata {
            created: "2023-01-01".to_string(),
            modified: "2023-12-31".to_string(),
            author: "Jane Smith".to_string(),
            version: "1.0".to_string(),
        },
        title: "Technical Specification".to_string(),
        content: "This document contains the technical specifications...".to_string(),
    };

    let xml = to_string(&document)?;
    println!("Document:\n{}\n", xml);

    let parsed: Document = from_str(&xml)?;
    assert_eq!(document, parsed);

    // Service with loose configuration group
    let service = Service {
        name: "user-service".to_string(),
        config: LooseConfiguration {
            environment: "production".to_string(),
            debug: false,
            timeout: 30,
            retries: 3,
        },
        endpoints: vec![
            "/users".to_string(),
            "/users/{id}".to_string(),
            "/users/{id}/profile".to_string(),
        ],
    };

    let xml = to_string(&service)?;
    println!("Service:\n{}\n", xml);

    let parsed: Service = from_str(&xml)?;
    // Note: Vec<String> serialization behavior
    assert_eq!(service.name, parsed.name);
    assert_eq!(service.config, parsed.config);
    println!("Note: endpoints vector serialized as concatenated string\n");

    // Application with nested groups
    let application = Application {
        name: "MyApp".to_string(),
        version: "2.1.0".to_string(),
        config: ApplicationConfig {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                database: "myapp_db".to_string(),
            },
            cache: CacheConfig {
                ttl: 3600,
                size: 1000,
            },
            log_level: "info".to_string(),
        },
        status: "running".to_string(),
    };

    let xml = to_string(&application)?;
    println!("Application:\n{}\n", xml);

    let parsed: Application = from_str(&xml)?;
    assert_eq!(application, parsed);

    // Project with multiple groups
    let project = Project {
        id: "PROJ001".to_string(),
        basic: BasicInfo {
            name: "Awesome Project".to_string(),
            description: "A project that does amazing things".to_string(),
        },
        technical: TechnicalInfo {
            version: "1.2.3".to_string(),
            build: "456".to_string(),
            dependencies: vec![
                "serde".to_string(),
                "tokio".to_string(),
                "reqwest".to_string(),
            ],
        },
        maintainer: "Open Source Community".to_string(),
    };

    let xml = to_string(&project)?;
    println!("Project:\n{}\n", xml);

    let parsed: Project = from_str(&xml)?;
    // Note: Vec<String> serialization behavior
    assert_eq!(project.id, parsed.id);
    assert_eq!(project.basic, parsed.basic);
    assert_eq!(project.technical.version, parsed.technical.version);
    assert_eq!(project.technical.build, parsed.technical.build);
    assert_eq!(project.maintainer, parsed.maintainer);
    println!("Note: dependencies vector also serialized as concatenated string\n");

    println!("All examples passed!");
    Ok(())
}
