Derives the [`Serialize`] trait for a type.

This macro works to serialize to XML-elements and other types of nodes including text and CDATA.
To serialize to attributes, use the [`SerializeAttribute`] derive macro instead.

<div style="background:rgba(120,145,255,0.45);padding:0.75em;">
<strong>NOTE:</strong> It is perfectly possible to derive both Serialize and SerializeAttribute for the same type, allowing the parent to decide which serialization method to use. Since deserialization can work from multiple sources, simply deriving Deserialize is sufficient to deserialize from either elements or attributes (depending on what is enabled through the derive macro).
</div>

## Modes of serialization

Modes of serialization depends on the type of data structure you are serializing.

### Serialize as an element - Structs with ` #[xelement(...)]` on the root of a type

The `#[xelement(...)]` attribute can be applied to the root of a type to specify that the type should be serialized as an element.

#### Root options

<table style="width:100%;">
<thead>
<tr>
<th>Name</th>
<th>Type</th>
<th>Description</th>
</tr>
</thead>
<tbody style="vertical-align:top;">
<!--=================================================-->
<tr>
<th>
name
</th>
<td>
<code>String</code>
</td>
<td>
Element name.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
namespace
</th>
<td>
<code>String</code>
</td>
<td>
Must be a valid namespace string. Exclusive with `namespace_expr`.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
namespace_expr
</th>
<td>
<code>Expr</code>
</td>
<td>
Element namespace expression. This should be a value of type `xmlity::XmlNamespace`. Exclusive with `namespace`.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
preferred_prefix
</th>
<td>
<code>String</code>
</td>
<td>
Must be a valid XML prefix.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
enforce_prefix
</th>
<td>
<code>bool</code>
</td>
<td>
Enforce the use of the preferred prefix. If this is set to `true`, the preferred prefix will be used even if it there is already a prefix bound to the namespace.
</td>
</tr>
<!--=================================================-->
</tbody>
</table>

#### Examples

##### Simple element

<table style="width:100%;">
<thead>
<tr>
<th>XML</th>
<th>Rust types</th>
</tr>
</thead>
<tbody style="vertical-align:top;">
<tr>
<td>

```xml
<note>
    <to>Tove</to>
    <from>Jani</from>
    <heading>Reminder</heading>
    <body>Message...</body>
</note>
```

</td>
<td rowspan="3">

```rust ignore
#[derive(Serialize)]
#[xelement(name = "to")]
struct To(String);

#[derive(Serialize)]
#[xelement(name = "from")]
struct From(String);

#[derive(Serialize)]
#[xelement(name = "heading")]
struct Heading(String);

#[derive(Serialize)]
#[xelement(name = "body")]
struct Body(String);

#[derive(Serialize)]
#[xelement(name = "note")]
struct Note {
    to: To,
    from: From,
    heading: Heading,
    body: Body,
}
```

</td>
</tr>
<tr>
<th>Rust value</th>
</tr>
<tr>
<td>

```rust ignore
Note {
    to: To("Tove".to_string()),
    from: From("Jani".to_string()),
    heading: Heading("Reminder".to_string()),
    body: Body("Message...".to_string()),
}
```

</td>
</tr>
</tbody>
</table>

##### Simple element with inline declarations

<table style="width:100%;">
<thead>
<tr>
<th>XML</th>
<th>Rust types</th>
</tr>
</thead>
<tbody style="vertical-align:top;">
<tr>
<td>

```xml
<note>
    <to>Tove</to>
    <from>Jani</from>
    <heading>Reminder</heading>
    <body>Message...</body>
</note>
```

</td>
<td rowspan="3">

```rust ignore
#[derive(Serialize)]
#[xelement(name = "note")]
struct Note {
    #[xelement(name = "to")]
    to: String,
    #[xelement(name = "from")]
    from: String,
    #[xelement(name = "heading")]
    heading: String,
    #[xelement(name = "body")]
    body: String,
}
```

</td>
</tr>
<tr>
<th>Rust value</th>
</tr>
<tr>
<td>

```rust ignore
Note {
    to: To("Tove".to_string()),
    from: From("Jani".to_string()),
    heading: Heading("Reminder".to_string()),
    body: Body("Message...".to_string()),
}
```

</td>
</tr>
</tbody>
</table>

### Serialize as a sequence - structs with `#[xvalue(...)]` on the root of a type or no root attribute

The `#[xvalue(...)]` attribute can be applied to the root of a type to specify that the type should be serialized as a sequence of values, where each field is serialized as a value.

#### Root options

<table style="width:100%;">
<thead>
<tr>
<th>Name</th>
<th>Type</th>
<th>Description</th>
</tr>
</thead>
<tbody style="vertical-align:top;">
<!--=================================================-->
<tr>
<th>
value
</th>
<td>
<code>String</code>
</td>
<td>
If the type is a unit struct, this attribute can be used to specify a text value to be serialized.
</td>
</tr>
<!--=================================================-->
</tbody>
</table>

#### Examples

##### Struct containing a sequence of elements and text

<table style="width:100%;">
<thead>
<tr>
<th>XML</th>
<th>Rust types</th>
</tr>
</thead>
<tbody style="vertical-align:top;">
<tr>
<td>

```xml
<name>Alice</name>
<name>Bob</name>
Text
```

</td>
<td rowspan="3">

```rust ignore
#[derive(Serialize)]
#[xelement(name = "name")]
struct Name {
    value: String,
}

#[derive(Serialize)]
struct NamesAndText {
    names: Vec<Name>,
    text: String,
}
```

</td>
</tr>
<tr>
<th>Rust value</th>
</tr>
<tr>
<td>

```rust ignore
NamesAndText {
    names: vec![
        Name { value: "Alice".to_string() },
        Name { value: "Bob".to_string() }
    ],
    text: vec!["Text".to_string()],
}
```

</td>
</tr>
</tbody>
</table>

<!--=================================================-->

### Serialize as one of several types - enums with `#[xvalue(...)]` on the root of a type or no root attribute

The `#[xvalue(...)]` attribute can be applied to the root of an enum to specify that the type should be serialized as a one of several types.

#### Root options

<table style="width:100%;">
<thead>
<tr>
<th>Name</th>
<th>Type</th>
<th>Description</th>
</tr>
</thead>
<tbody style="vertical-align:top;">
<!--=================================================-->
<tr>
<th>
rename_all
</th>
<td>
<code>"lowercase"</code>, <code>"UPPERCASE"</code>, <code>"PascalCase"</code>, <code>"camelCase"</code>, <code>"snake_case"</code>, <code>"SCREAMING_SNAKE_CASE"</code>, <code>"kebab-case"</code>, <code>"SCREAMING-KEBAB-CASE"</code>
</td>
<td>
Decides what format to use for the serialized unit variants if they don't have values specified. 
</td>
</tr>
<!--=================================================-->
<tr>
<th>
serialization_format
</th>
<td>
<code>text</code>, <code>cdata</code>
</td>
<td>
Decides to what form the value should be serialized.
</td>
</tr>
<!--=================================================-->
</tbody>
</table>

#### Variant options

Variants have the same options as struct roots, and indeed work the same way.

#### Examples

##### Enum with just text values

<table style="width:100%;">
<thead>
<tr>
<th>XML</th>
<th>Rust types</th>
</tr>
</thead>
<tbody style="vertical-align:top;">
<tr>
<td>

```xml
Jani
```

</td>
<td rowspan="3">

```rust ignore
#[derive(Serialize)]
enum Name {
    #[xvalue(value = "Jani")]
    Jani,
    #[xvalue(value = "Tove")]
    Tove,
}
```

</td>
</tr>
<tr>
<th>Rust value</th>
</tr>
<tr>
<td>

```rust ignore
Name::Jani
```

</td>
</tr>
</tbody>
</table>

<!--=================================================-->

##### Enum that is either a float or a string

<table style="width:100%;">
<thead>
<tr>
<th>XML</th>
<th>Rust value</th>
<th>Rust types</th>
</tr>
</thead>
<tbody style="vertical-align:top;">
<tr>
<td>

```xml
1.0
```

</td>
<td>

```rust ignore
FloatOrString::Float(1.0)
```

</td>
<td rowspan="3">

```rust ignore
#[derive(Serialize)]
enum FloatOrString {
    Float(f64),
    String(String),
}
```

</td>
</tr>
<tr>
<td>

```xml
Not float
```

</td>
<td>

```rust ignore
FloatOrString::String("Not float".to_string())
```

</td>
</tr>
</tbody>
</table>
<!--=================================================-->
