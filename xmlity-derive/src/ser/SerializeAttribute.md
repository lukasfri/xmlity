Derives the [`SerializeAttribute`] trait for a type.

This macro works to serialize to XML-attributes.
To serialize to elements, use the [`Serialize`] derive macro instead.

<div style="background:rgba(120,145,255,0.45);padding:0.75em;">
<strong>NOTE:</strong> It is perfectly possible to derive both Serialize and SerializeAttribute for the same type, allowing the parent to decide which serialization method to use. Since deserialization can work from multiple sources, simply deriving Deserialize is sufficient to deserialize from either elements or attributes (depending on what is enabled through the derive macro).
</div>

To configure the serialization, use the `#[xattribute(...)]` attribute on the root of the type. This attribute is required.

## Configuration

### Serialize as an attribute - `#[xattribute(...)]` on the root of a type

<table style="width:100%;">
<thead>
<tr><th colspan="2">

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
