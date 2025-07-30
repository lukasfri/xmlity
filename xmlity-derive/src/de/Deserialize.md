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
<code>"any"</code>, <code>"at_end"</code>, <code>"none"</code>
</td>
<td>
Allow unknown children when deserializing.<br/>
- <code>"any"</code>: Allow any unknown children.<br/>
- <code>"at_end"</code> (default): Allow unknown children only at the end of the element.<br/>
- <code>"none"</code>: Do not allow unknown children at all.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
allow_unknown_attributes
</th>
<td>
<code>"any"</code>, <code>"at_end"</code>, <code>"none"</code>
</td>
<td>
Allow unknown attributes when deserializing.<br/>
- <code>"any"</code>: Allow any unknown attributes.<br/>
- <code>"at_end"</code> (default): Allow unknown attributes only at the end of the element.<br/>
- <code>"none"</code>: Do not allow unknown attributes at all.
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
Allow any name for the element when deserializing.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
attribute_order
</th>
<td>
<code>"strict"</code>, <code>"none"</code>
</td>
<td>
Set if the order of attributes is important when deserializing.<br/>
- <code>"strict"</code>: The order of attributes must match the order in the struct or enum variant.<br/>
- <code>"none"</code> (default): The order of attributes does not matter, but the attributes must be present.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
children_order
</th>
<td>
<code>"strict"</code>, <code>"none"</code>
</td>
<td>
Set if the order of children is important when deserializing.<br/>
- <code>"strict"</code>: The order of children must match the order in the struct or enum variant.<br/>
- <code>"none"</code> (default): The order of children does not matter, but the children must be present.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
ignore_whitespace
</th>
<td>
<code>"any"</code>, <code>"none"</code>
</td>
<td>
Set if whitespace should be ignored when deserializing.<br/>
- <code>"any"</code> (default): Ignore any whitespace.<br/>
- <code>"none"</code>: Do not ignore whitespace.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
ignore_comments
</th>
<td>
<code>"any"</code>, <code>"none"</code>
</td>
<td>
Set if comments should be ignored when deserializing.<br/>
- <code>"any"</code> (default): Ignore any comments.<br/>
- <code>"none"</code>: Do not ignore comments.
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
The text value to serialize to and deserialize from. If the type is a unit struct, this attribute can be used to specify a text value to deserialize from.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
ignore_whitespace
</th>
<td>
<code>"any"</code>, <code>"none"</code>
</td>
<td>
Set if whitespace should be ignored when deserializing.<br/>
- <code>"any"</code> (default): Ignore any whitespace.<br/>
- <code>"none"</code>: Do not ignore whitespace.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
ignore_comments
</th>
<td>
<code>"any"</code>, <code>"none"</code>
</td>
<td>
Set if comments should be ignored when deserializing.<br/>
- <code>"any"</code> (default): Ignore any comments.<br/>
- <code>"none"</code>: Do not ignore comments.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
allow_unknown
</th>
<td>
<code>"any"</code>, <code>"at_end"</code>, <code>"none"</code>
</td>
<td>
Allow unknown values when deserializing.<br/>
- <code>"any"</code>: Allow any unknown values.<br/>
- <code>"at_end"</code> (default): Allow unknown values only at the end of the element.<br/>
- <code>"none"</code>: Do not allow unknown values at all.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
order
</th>
<td>
<code>"strict"</code>, <code>"none"</code>
</td>
<td>
Set if the order of values is important when deserializing.<br/>
- <code>"strict"</code>: The order of values must match the order in the struct or enum variant.<br/>
- <code>"none"</code> (default): The order of values does not matter, but the values must be present.
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
Attribute name. If not specified, the name of the struct is used.
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
The namespace of the attribute, defined as a string. This is exclusive with <code>namespace_expr</code>. If none of these are specified, the absence of a namespace is assumed. Must be a valid namespace string.
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
The namespace of the attribute given as an expression to an <code>xmlity::XmlNamespace</code> value. This is exclusive with <code>namespace</code>. If none of these are specified, the absence of a namespace is assumed.
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
The preferred prefix for the attribute, defined as a string. This is exclusive with <code>enforce_prefix</code>. If none of these are specified, the absence of a prefix is assumed. Must be a valid XML prefix. (Serialize only)
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
Always set the prefix of the attribute to the prefix set in <code>preferred_prefix</code>. (Serialize only)
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
Allow any name for the attribute when deserializing.
</td>
</tr>
<!--=================================================-->
</tbody>
</table>

### Deserialize as one of several types - enums with `#[xvalue(...)]` on the root of a type or no root attribute

The `#[xvalue(...)]` attribute can be applied to the root of an enum to specify that the type can be deserialized to one of several types.

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
The text casing to use for unit variants when deserializing if they don't have values specified.
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
