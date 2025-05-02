Derives the [`Deserialize`] trait for a type.

This macro supports deriving deserialization from elements, attributes and values.

One of the following can be applied to the root of a type:

- `#[xelement(...)]` - Specifies that the type can be deserialized as an element.
- `#[xvalue(...)]` - Specifies that the type can be deserialized as a value.
- `#[xattribute(...)]` - Specifies that the type can be deserialized as an attribute.
- No attribute/default behavior - Specifies that the type is a composite type. Can be deserialized from a sequence of elements.

## Modes of deserialization

Modes of deserialization depends on the type of data structure you are deserializing.

### Deserialize as an element - `#[xelement(...)]` on the root of a type

The `#[xelement(...)]` attribute can be applied to the root of a type to specify that the type can be deserialized from an element.

#### Root Options

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
allow_unknown_children
</th>
<td>
<code>bool</code>
</td>
<td>
Element namespace.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
allow_unknown_attributes
</th>
<td>
<code>bool</code>
</td>
<td>
Element namespace.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
deserialize_any_name
</th>
<td>
<code>bool</code>
</td>
<td>
Element namespace.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
attribute_order
</th>
<td>
<code>"loose"</code>, <code>"none"</code>
</td>
<td>
Element namespace.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
children_order
</th>
<td>
<code>"loose"</code>, <code>"none"</code>
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
#[derive(Deserialize)]
#[xelement(name = "from")]
struct From(String);

#[derive(Deserialize)]
#[xelement(name = "heading")]
struct Heading(String);

#[derive(Deserialize)]
#[xelement(name = "body")]
struct Body(String);

#[derive(Deserialize)]
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

### Deserialize from a sequence - structs with `#[xvalue(...)]` on the root of a type or no root attribute

The `#[xvalue(...)]` attribute can be applied to the root of a type to specify that the type can be deserialized from a text or CDATA node.

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
Decides how enums should be deserialized.
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
Decides in what form the value should be deserialized from.
</td>
</tr>
<!--=================================================-->
</tbody>
</table>

### Deserialize from an attribute - structs with `#[xattribute(...)]` on the root of a type

The `#[xattribute(...)]` attribute can be applied to the root of a type to specify that the type can be deserialized from an attribute.

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
deserialize_any_name
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

### Deserialize as one of several types - enums with `#[xvalue(...)]` on the root of a type or no root attribute
