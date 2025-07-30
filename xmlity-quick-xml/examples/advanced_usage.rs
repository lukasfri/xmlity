//! Advanced XMLity Usage Examples
//!
//! Demonstrates advanced xmlity features:
//! - Serialization groups (#[xgroup])
//! - Element and value enums
//! - Optional fields and defaults
//! - HashMap serialization
//! - Nested structures
//! - Namespace handling for inline elements
//!
//! Run with: `cargo run --example advanced_usage`

use std::collections::HashMap;
use xmlity::{DeserializationGroup, Deserialize, SerializationGroup, Serialize};
use xmlity_quick_xml::{from_str, to_string};

// Main project structure
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "project", namespace = "http://example.com/projects")]
struct Project {
    #[xattribute(name = "id")]
    id: String,

    #[xattribute(name = "version")]
    version: String,

    #[xelement(name = "metadata", namespace = "http://example.com/projects")]
    metadata: ProjectMetadata,

    #[xelement(name = "dependency", namespace = "http://example.com/projects")]
    dependencies: Vec<Dependency>,

    #[xelement(name = "build", namespace = "http://example.com/projects")]
    build: BuildConfig,

    #[xelement(name = "notifications", namespace = "http://example.com/projects")]
    notifications: NotificationSettings,
}

mod keyword_container_with {
    use xmlity::{Deserialize, Serialize};

    use crate::KeywordContainer;

    pub fn serialize<S>(keywords: &KeywordContainer, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: xmlity::Serializer,
    {
        keywords
            .0
            .iter()
            .map(|keyword| keyword.trim())
            .collect::<Vec<_>>()
            .join(",")
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<KeywordContainer, D::Error>
    where
        D: xmlity::Deserializer<'de>,
    {
        let keywords_str: String = String::deserialize(deserializer)?;
        let keywords = keywords_str
            .split(",")
            .map(str::trim)
            .map(String::from)
            .collect();

        Ok(KeywordContainer(keywords))
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(with = "keyword_container_with")]
struct KeywordContainer(Vec<String>);

// Serialization group - fields are serialized without wrapper element
#[derive(Debug, PartialEq, Serialize, Deserialize, SerializationGroup, DeserializationGroup)]
#[xgroup]
struct ProjectMetadata {
    #[xelement(name = "name")]
    name: String,

    #[xelement(name = "description")]
    description: String,

    #[xelement(name = "license")]
    license: License,

    #[xelement(name = "keywords")]
    keywords: KeywordContainer,

    #[xelement(name = "repository", optional = true)]
    repository: Option<Repository>,
}

// Element enum - each variant becomes a different XML element
#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum License {
    #[xelement(name = "mit")]
    Mit,

    #[xelement(name = "apache")]
    Apache {
        #[xattribute(name = "version")]
        version: String,
    },

    #[xelement(name = "custom")]
    Custom {
        #[xelement(name = "name")]
        name: String,

        #[xelement(name = "url")]
        url: String,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "repository", namespace = "http://example.com/projects")]
struct Repository {
    #[xattribute(name = "url")]
    url: String,

    #[xelement(name = "branch", default)]
    default_branch: String,
}

impl Default for Repository {
    fn default() -> Self {
        Self {
            url: "".to_string(),
            default_branch: "main".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "dependency", namespace = "http://example.com/projects")]
struct Dependency {
    #[xattribute(name = "name")]
    name: String,

    #[xattribute(name = "version")]
    version: String,

    #[xelement(name = "features", optional = true)]
    features: Option<Vec<String>>,

    // Value enum - serialized as element text content
    #[xvalue(default)]
    dep_type: DependencyType,
}

// Value enum - serialized as element text content
#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[xvalue(rename_all = "lowercase")]
enum DependencyType {
    #[default]
    Runtime,
    Development,
    Build,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "build", namespace = "http://example.com/projects")]
struct BuildConfig {
    #[xattribute(name = "tool")]
    tool: String,

    // HashMap serialization as key-value pairs
    #[xelement(name = "environment")]
    environment: HashMap<String, String>,

    #[xelement(name = "targets")]
    targets: Vec<BuildTarget>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "target", namespace = "http://example.com/projects")]
struct BuildTarget {
    #[xattribute(name = "platform")]
    platform: String,

    #[xvalue(default)]
    optimization: OptimizationLevel,
}

// Value enum with custom names
#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[xvalue(rename_all = "lowercase")]
enum OptimizationLevel {
    #[default]
    Debug,
    Release,
    #[xvalue(value = "size-opt")]
    Size,
    #[xvalue(value = "speed-opt")]
    Speed,
}

// Notification channels with different types
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "notifications", namespace = "http://example.com/projects")]
struct NotificationSettings {
    #[xelement(name = "enabled")]
    enabled: bool,

    #[xelement(name = "channel")]
    channels: Vec<NotificationChannel>,
}

// Each enum variant becomes its own element
#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum NotificationChannel {
    #[xelement(name = "email")]
    Email {
        #[xattribute(name = "address")]
        address: String,

        #[xelement(name = "format")]
        format: EmailFormat,
    },

    #[xelement(name = "slack")]
    Slack {
        #[xattribute(name = "webhook")]
        webhook_url: String,

        #[xelement(name = "mention")]
        mention_users: Vec<String>,
    },

    #[xelement(name = "webhook")]
    Webhook {
        #[xattribute(name = "url")]
        url: String,

        #[xelement(name = "headers")]
        headers: HashMap<String, String>,

        #[xelement(name = "auth")]
        auth: WebhookAuth,
    },
}

// Element enum for authentication methods
#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum WebhookAuth {
    #[xelement(name = "none")]
    None,

    #[xelement(name = "bearer")]
    Bearer {
        #[xattribute(name = "token")]
        token: String,
    },

    #[xelement(name = "basic")]
    Basic {
        #[xattribute(name = "username")]
        username: String,

        #[xattribute(name = "password")]
        password: String,
    },
}

// Value enum for email formats
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(rename_all = "lowercase")]
enum EmailFormat {
    Plain,
    Html,
    Markdown,
}

fn create_sample_project() -> Project {
    let mut env = HashMap::new();
    env.insert("RUST_LOG".to_string(), "info".to_string());
    env.insert(
        "DATABASE_URL".to_string(),
        "postgresql://localhost/app".to_string(),
    );

    let mut webhook_headers = HashMap::new();
    webhook_headers.insert("Content-Type".to_string(), "application/json".to_string());
    webhook_headers.insert("User-Agent".to_string(), "xmlity-example/1.0".to_string());

    Project {
        id: "xmlity-example".to_string(),
        version: "1.0.0".to_string(),
        metadata: ProjectMetadata {
            name: "XMLity Advanced Example".to_string(),
            description: "Demonstrates advanced xmlity serialization patterns".to_string(),
            license: License::Apache {
                version: "2.0".to_string(),
            },
            keywords: KeywordContainer(vec![
                "rust".to_string(),
                "xml".to_string(),
                "serialization".to_string(),
            ]),
            repository: Some(Repository {
                url: "https://github.com/example/xmlity-example".to_string(),
                default_branch: "main".to_string(),
            }),
        },
        dependencies: vec![
            Dependency {
                name: "serde".to_string(),
                version: "1.0".to_string(),
                features: Some(vec!["derive".to_string()]),
                dep_type: DependencyType::Runtime,
            },
            Dependency {
                name: "tokio".to_string(),
                version: "1.0".to_string(),
                features: Some(vec!["rt-multi-thread".to_string(), "macros".to_string()]),
                dep_type: DependencyType::Runtime,
            },
            Dependency {
                name: "pretty_assertions".to_string(),
                version: "1.0".to_string(),
                features: None,
                dep_type: DependencyType::Development,
            },
        ],
        build: BuildConfig {
            tool: "cargo".to_string(),
            environment: env,
            targets: vec![
                BuildTarget {
                    platform: "linux".to_string(),
                    optimization: OptimizationLevel::Release,
                },
                BuildTarget {
                    platform: "windows".to_string(),
                    optimization: OptimizationLevel::Release,
                },
            ],
        },
        notifications: NotificationSettings {
            enabled: true,
            channels: vec![
                NotificationChannel::Email {
                    address: "team@example.com".to_string(),
                    format: EmailFormat::Html,
                },
                NotificationChannel::Slack {
                    webhook_url: "https://hooks.slack.com/webhook".to_string(),
                    mention_users: vec!["@channel".to_string()],
                },
                NotificationChannel::Webhook {
                    url: "https://api.example.com/notify".to_string(),
                    headers: webhook_headers,
                    auth: WebhookAuth::Bearer {
                        token: "secret-token".to_string(),
                    },
                },
            ],
        },
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Advanced XMLity Examples ===\n");

    let project = create_sample_project();

    // Serialize
    let xml = to_string(&project)?;

    // Show partial XML output
    println!(
        "Project XML (first 1200 chars):\n{}\n",
        if xml.len() > 1200 {
            format!("{}...", &xml[..1200])
        } else {
            xml.clone()
        }
    );

    // Deserialize
    let parsed: Project = from_str(&xml)?;

    // Verify
    assert_eq!(project.id, parsed.id);
    assert_eq!(project.version, parsed.version);
    assert_eq!(project.metadata.name, parsed.metadata.name);
    assert_eq!(project.metadata.description, parsed.metadata.description);
    assert_eq!(project.dependencies.len(), parsed.dependencies.len());

    // Show results
    println!("Project: {}", parsed.metadata.name);
    println!("Version: {}", parsed.version);
    println!("Dependencies: {}", parsed.dependencies.len());
    println!("Build Targets: {}", parsed.build.targets.len());
    println!(
        "Notification Channels: {}",
        parsed.notifications.channels.len()
    );

    // Metadata group example
    println!("\n--- Metadata Group ---");
    let metadata_xml = to_string(&project.metadata)?;
    println!("Metadata XML:\n{}\n", metadata_xml);

    let parsed_metadata: ProjectMetadata = from_str(&metadata_xml)?;
    assert_eq!(project.metadata.name, parsed_metadata.name);

    // Enum examples
    println!("--- Enum Examples ---");
    let license_xml = to_string(&project.metadata.license)?;
    println!("License: {}", license_xml);

    let dep_type_xml = to_string(&project.dependencies[0].dep_type)?;
    println!("Dependency type: {}", dep_type_xml);

    println!("\nAll examples completed successfully!");

    Ok(())
}

// Key patterns demonstrated:
// - Serialization groups: Fields without wrapper elements
// - Element enums: Variants as different XML elements
// - Value enums: Text content with rename rules
// - Collections: Vec and HashMap serialization
// - Optional fields and defaults
// - Namespace handling: Inline elements inherit context from parents
