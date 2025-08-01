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
The namespace of the element, defined as a string. This is exclusive with <code>namespace_expr</code>. If none of these are specified, the absence of a namespace is assumed. Must be a valid namespace string.
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
The namespace of the element given as an expression to an <code>xmlity::XmlNamespace</code> value. This is exclusive with <code>namespace</code>. If none of these are specified, the absence of a namespace is assumed.
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
The element is serialized with the given prefix. Must be a valid XML prefix.
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
Always set the prefix of the element to the prefix set in <code>preferred_prefix</code>. Enforce the use of the preferred prefix. If this is set to <code>true</code>, the preferred prefix will be used even if there is already a prefix bound to the namespace.
</td>
</tr>
<!--=================================================-->
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
The text casing to use for unit variants when serializing if they don't have values specified.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
with
</th>
<td>
<code>Path</code>
</td>
<td>
The path to the module that provides the serialization and deserialization functions. <code>::serialize</code> and <code>::deserialize</code> will be appended to this path and used as the <code>serialize_with</code> and <code>deserialize_with</code> functions.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
serialize_with
</th>
<td>
<code>Expr</code>
</td>
<td>
Use function to serialize the value. Should have signature like <code>pub fn serialize&lt;S: xmlity::Serializer&gt;(value: &T, serializer: S) -> Result&lt;S::Ok, S::Error&gt;</code>
</td>
</tr>
<!--=================================================-->
<tr>
<th>
deserialize_with
</th>
<td>
<code>Expr</code>
</td>
<td>
Use function to deserialize the value. Should have signature like <code>fn deserialize&lt;'de, D: xmlity::Deserializer&lt;'de&gt;&gt;(deserializer: D) -> Result&lt;T, D::Error&gt;</code>
</td>
</tr>
<!--=================================================-->
</tbody>
</table>

#### Variant options

Variants have the same options as struct roots, and indeed work the same way.
