Derives the [`SerializationGroup`] trait for a type.

To configure the serialization, use the `#[xgroup(...)]` attribute on the root of the type.

<div style="background:rgba(120,145,255,0.45);padding:0.75em;">
<strong>NOTE:</strong> This trait/attribute is not mutually exclusive with the Serialize trait/attributes. This means that you could for example use a struct both as a sequence (Serialize with no attribute) and as a group (SerializationGroup with the attribute).
</div>

## Configuration

### #[xgroup(...)]

#### Options

None for serialization currently.
