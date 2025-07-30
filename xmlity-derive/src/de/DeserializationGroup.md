Derives the [`DeserializationGroup`] trait for a type.

To configure the deserialization, use the `#[xgroup(...)]` attribute on the root of the type.

<div style="background:rgba(120,145,255,0.45);padding:0.75em;">
<strong>NOTE:</strong> This trait/attribute is not mutually exclusive with the [Deserialize] trait/attribute. This means that you could for example use a struct both as a sequence ([Deserialize] with no attribute) and as a group ([DeserializationGroup] with the attribute).
</div>

[Deserialize]: Deserialize
[DeserializationGroup]: DeserializationGroup

## Configuration

### Deserialize a part of an element - `#[xgroup(...)]` on the root of a type

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
Set if the order of attributes is important when deserializing.<br/>
- <code>"strict"</code>: The attributes must come directly after each other, and this group will try to deserialize them in one go.<br/>
- <code>"loose"</code>: The order of attributes must come relative to each other, but they can be separated by other attributes outside this group.<br/>
- <code>"none"</code> (default): The order of attributes is not important.
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
Set if the order of children is important when deserializing.<br/>
- <code>"strict"</code>: The children must come directly after each other, and this group will try to deserialize them in one go.<br/>
- <code>"loose"</code>: The order of children must come relative to each other, but they can be separated by other children outside this group.<br/>
- <code>"none"</code> (default): The order of children is not important.
</td>
</tr>
<!--=================================================-->
</tbody>
</table>
