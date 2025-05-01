Derives the [`Serialize`] trait for a type.

This macro works to serialize to XML-elements and other types of nodes including text and CDATA.
To serialize to attributes, use the [`SerializeAttribute`] derive macro instead.

<div style="background:rgba(120,145,255,0.45);padding:0.75em;">
<strong>NOTE:</strong> It is perfectly possible to derive both Serialize and SerializeAttribute for the same type, allowing the parent to decide which serialization method to use. Since deserialization can work from multiple sources, simply deriving Deserialize is sufficient to deserialize from either elements or attributes (depending on what is enabled through the derive macro).
</div>

ONE OF the following attributes can be applied to the root of a type to specify how the type should be serialized:

- `#[xelement(...)]` - Specifies that the type should be serialized as an element.
- `#[xvalue(...)]` - Specifies that the type should be serialized as a value.
- None (default) - Specifies that the type is a composite type. Currently, this is only used for enums which allow for one of the variants to be serialized as an element or value.

## Configuration

### #[xelement(...)]

The `#[xelement(...)]` attribute can be applied to the root of a type to specify that the type should be serialized as an element.

#### Options

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
Must be a valid namespace string.
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
Element namespace expression. This should be a value of type `xmlity::XmlNamespace`.
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
Element namespace.
</td>
</tr>
<!--=================================================-->
</tbody>
</table>

#### Examples

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

### #[xvalue(...)]

The `#[xvalue(...)]` attribute can be applied to the root of a type to specify that the type should be serialized as a value. What this means differs based on the type of the root.

- For enums, the enum will be serialized as a value, with the variant name (or specified name) as the value.

#### Options

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
Decides how enums should be serialized.
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
Decides in what form the value should be serialized.
</td>
</tr>
<!--=================================================-->
</tbody>
</table>

### No root attribute

If no root attribute is specified, the root will be serialized as a container with no individual serialization taking place. Instead it will defer to the fields of the root.

- For structs, the fields will be serialized as a sequence of elements.
- For enums, the active variant will be serialized as a sequence of elements.
