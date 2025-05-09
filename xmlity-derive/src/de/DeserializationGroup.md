Derives the [`DeserializationGroup`] trait for a type.

To configure the deserialization, use the `#[xgroup(...)]` attribute on the root of the type.

<div style="background:rgba(120,145,255,0.45);padding:0.75em;">
<strong>NOTE:</strong> This trait/attribute is not mutually exclusive with the [Deserialize] trait/attribute. This means that you could for example use a struct both as a sequence ([Deserialize] with no attribute) and as a group ([DeserializationGroup] with the attribute).
</div>

[Deserialize]: Deserialize
[DeserializationGroup]: DeserializationGroup

## Configuration

### Serialize a part of an element - `#[xgroup(...)]` on the root of a type

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
attribute_order
</th>
<td>
<code>"strict"</code>, <code>"loose"</code>, <code>"none"</code>
</td>
<td>
The accepted order of attributes.
- <code>"strict"</code> - The attributes must be in the same order as the fields with no fields between them.
- <code>"loose"</code> - The attributes must be in the same order as the fields, but there can be fields between them.
- <code>"none"</code> - The attributes can be in any order.
</td>
</tr>
<!--=================================================-->
<tr>
<th>
children_order
</th>
<td>
<code>"strict"</code>, <code>"loose"</code>, <code>"none"</code>
</td>
<td>
The accepted order of children.
- <code>"strict"</code> - The children must be in the same order as the fields with no fields between them.
- <code>"loose"</code> - The children must be in the same order as the fields, but there can be fields between them.
- <code>"none"</code> - The children can be in any order.
</td>
</tr>
<!--=================================================-->
</tbody>
</table>
