# XMLity-Quick-XML Examples

This directory contains comprehensive examples demonstrating all the different options and features available in `xmlity-derive`. Each example focuses on specific aspects of XML serialization and deserialization using the XMLity framework.

## Examples Overview

### 1. [Basic Elements](basic_elements.rs)

**Core concepts and fundamental usage**

- Basic struct serialization/deserialization with `#[xelement]`
- Element naming with `name` attribute
- Namespace declarations with `namespace` attribute
- Text content handling with `#[xvalue]`

### 2. [Attributes](attributes.rs)

**Working with XML attributes**

- `SerializeAttribute` trait for standalone attributes
- Inline attribute declarations with `#[xattribute]`
- Deferred attributes with `deferred = true`
- Attribute namespaces and prefixes
- Optional attributes with `optional = true`

### 3. [Enums and Values](enums_and_values.rs)

**Enum serialization and value types**

- Enum serialization with `#[xvalue]` and `rename_all` options
- Different text casing formats (lowercase, kebab-case, SCREAMING_SNAKE_CASE, etc.)
- Custom enum values with `value` attribute
- Complex enums with different serialization modes
- Mixed element and value enum variants

### 4. [Field Options](field_options.rs)

**Field-level configuration options**

- Default values with `default` and `default_with`
- Extendable fields for merging multiple elements
- Conditional serialization with `skip_serializing_if`
- Optional fields and `Option<T>` types
- Combining multiple field options

### 5. [Namespaces and Prefixes](namespaces_and_prefixes.rs)

**XML namespace handling**

- Namespace declarations on elements and attributes
- Namespace expressions using `xmlity::XmlNamespace`
- Preferred prefixes with `preferred_prefix`
- Prefix enforcement with `enforce_prefix = true`
- Mixed namespace scenarios in complex documents

### 6. [Ordering and Unknown Content](ordering_and_unknown.rs)

**Content ordering and flexibility**

- Attribute and children ordering constraints (`strict`, `loose`, `none`)
- Handling unknown content with `allow_unknown_attributes` and `allow_unknown_children`
- Whitespace and comment handling with `ignore_whitespace` and `ignore_comments`
- Flexible element names with `deserialize_any_name`

### 7. [Groups](groups.rs)

**SerializationGroup and DeserializationGroup**

- Basic group usage with `#[xgroup]`
- Group ordering options (strict, loose, none)
- Combining groups with regular elements
- Nested groups and complex structures
- Logical organization of related fields

### 8. [Custom Serialization](custom_serialization.rs)

**Custom serialization functions**

- Using `with`, `serialize_with`, and `deserialize_with` options
- Custom serialization modules for complex data types
- Date formatting, key-value maps, and comma-separated lists
- Combining multiple custom serialization approaches

### 9. [Advanced Usage](advanced_usage.rs)

**Real-world complex scenarios**

- Complex nested structures representing a software project
- Combining different serialization modes in one structure
- Performance considerations for large documents
- Best practices for organizing complex XML schemas
