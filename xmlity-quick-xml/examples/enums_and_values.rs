//! Enums and Value Types Examples
//!
//! Demonstrates enums and value types:
//! - Enum serialization with rename_all
//! - Custom enum values
//! - Complex enum variants

use xmlity::{Deserialize, Serialize};
use xmlity_quick_xml::{from_str, to_string};

// Simple enum with rename_all
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(rename_all = "lowercase")]
enum Status {
    Active,
    Inactive,
    Pending,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(rename_all = "kebab-case")]
enum Priority {
    VeryHigh,
    High,
    Medium,
    Low,
    VeryLow,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xvalue(rename_all = "SCREAMING_SNAKE_CASE")]
enum Category {
    UserInterface,
    BackendApi,
    Database,
    ThirdPartyIntegration,
}

// Enum with custom values
#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Color {
    #[xvalue(value = "red")]
    Red,
    #[xvalue(value = "green")]
    Green,
    #[xvalue(value = "blue")]
    Blue,
    #[xvalue(value = "custom-color")]
    Custom(String),
}

// Complex enum with different serialization modes
#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum TaskType {
    #[xelement(name = "bug-report")]
    BugReport {
        #[xattribute(name = "severity")]
        severity: String,
        #[xelement(name = "description")]
        description: String,
    },
    #[xelement(name = "feature-request")]
    FeatureRequest {
        #[xattribute(name = "priority")]
        priority: Priority,
        #[xelement(name = "requirements")]
        requirements: String,
    },
    #[xvalue]
    SimpleTask(String),
}

// Container structs
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "task")]
struct Task {
    #[xattribute(name = "id")]
    id: String,
    #[xattribute(name = "status")]
    status: Status,
    #[xelement(name = "title")]
    title: String,
    #[xelement(name = "category")]
    category: Category,
    #[xelement(name = "priority")]
    priority: Priority,
    task_type: TaskType,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[xelement(name = "theme")]
struct Theme {
    #[xattribute(name = "name")]
    name: String,
    #[xelement(name = "primary-color")]
    primary_color: Color,
    #[xelement(name = "secondary-color")]
    secondary_color: Color,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Enums and Value Types ===\n");

    // Task with bug report
    let bug_task = Task {
        id: "TASK-001".to_string(),
        status: Status::Active,
        title: "Fix login issue".to_string(),
        category: Category::UserInterface,
        priority: Priority::VeryHigh,
        task_type: TaskType::BugReport {
            severity: "critical".to_string(),
            description: "Users cannot log in to the application".to_string(),
        },
    };

    let xml = to_string(&bug_task)?;
    println!("Bug Task:\n{}\n", xml);

    let parsed: Task = from_str(&xml)?;
    assert_eq!(bug_task, parsed);

    // Task with feature request
    let feature_task = Task {
        id: "TASK-002".to_string(),
        status: Status::Pending,
        title: "Add dark mode".to_string(),
        category: Category::UserInterface,
        priority: Priority::Medium,
        task_type: TaskType::FeatureRequest {
            priority: Priority::High,
            requirements: "Implement dark theme for all UI components".to_string(),
        },
    };

    let xml = to_string(&feature_task)?;
    println!("Feature Task:\n{}\n", xml);

    let parsed: Task = from_str(&xml)?;
    assert_eq!(feature_task, parsed);

    // Task with simple task type
    let simple_task = Task {
        id: "TASK-003".to_string(),
        status: Status::Inactive,
        title: "Update documentation".to_string(),
        category: Category::BackendApi,
        priority: Priority::Low,
        task_type: TaskType::SimpleTask("Update API documentation with new endpoints".to_string()),
    };

    let xml = to_string(&simple_task)?;
    println!("Simple Task:\n{}\n", xml);

    let parsed: Task = from_str(&xml)?;
    assert_eq!(simple_task, parsed);

    // Theme with colors
    let theme = Theme {
        name: "Ocean".to_string(),
        primary_color: Color::Blue,
        secondary_color: Color::Custom("teal".to_string()),
    };

    let xml = to_string(&theme)?;
    println!("Theme:\n{}\n", xml);

    let parsed: Theme = from_str(&xml)?;
    assert_eq!(theme, parsed);

    println!("All examples passed!");
    Ok(())
}
