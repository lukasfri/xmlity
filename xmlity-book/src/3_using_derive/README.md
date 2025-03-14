# Derive details

## `Serialize`/`Deserialize` - `#[xelement(...)]` on structs

The `#[xelement(...)]` attribute can be used to serialize/deserialize a struct as an XML element.

### `name = "..."` and `namespace = "..."`

If `name` is not specified, the name of the struct will be used.
These attributes can be used to require that an element has a specific name and namespace. If the namespace is not specified, it will be defaulted to the blank namespace.

### `deserialize_any_name = true/false`

If `deserialize_any_name` is set to `true`, the element will be deserialized regardless of its name. This is useful for elements that can have multiple names, or for elements that are used in multiple contexts.

### `attribute_order = "..."` and `children_order = "..."`

By default, elements in XMLity are deserialized regardless of the order of the inputs, but this can be changed using the `attribute_order` and `children_order` attributes. These attributes change elements to require that the inputs be in the same order as the fields in the struct. The possible values are as follows:

- `loose`: Elements must be in order, but incorrect elements can be interspersed between them (if `allow_unknown_attributes`/`allow_unknown_children` is enabled).
- `none`: Elements can be in any order (default).

<!-- TODO: Some examples -->

### `preferred_prefix = "..."`

Thia field is used to specify the preferred prefix for the element. This is used when serializing the element to XML. If the element is not in the default namespace, the preferred prefix will be used. If the element is in the default namespace, the preferred prefix will be ignored.

### `enforce_prefix = true/false`

This field requires `preferred_prefix` to be specified. This field encorces that the element will always use the specified prefix when serialized to XML.

## `Serialize`/`Deserialize` - `#[xvalue(...)]` on enums

The `#[xvalue(...)]` attribute can be used to serialize/deserialize an enum as a text value.

### `rename_all = "..."`

This field is used to specify the format of the text value. The possible values are `"lowercase"`, `"UPPERCASE"`, `"PascalCase"`, `"camelCase"`, `"snake_case"`, `"SCREAMING_SNAKE_CASE"`, `"kebab-case"`, `"SCREAMING-KEBAB-CASE"`.

### `allow_cdata = true/false`

Defaults to true. If true, the text value can be deserialized from a CDATA section.

### `allow_text = true/false`

Defaults to true. If true, the text value can be deserialized from a text node.

## `SerializationGroup`/`DeserializationGroup` - `#[xgroup(...)]` on structs

The `SerializationGroup` and `DeserializationGroup` traits are used to define groups of elements that can be serialized and deserialized together. This is useful for elements that are always used together, or for elements that are part of a larger structure.

### `attribute_order = "..."` and `children_order = "..."`

By default, groups in XMLity are deserialized regardless of the order of the inputs, but this can be changed using the `attribute_order` and `children_order` attributes. These attributes change groups to require that the inputs be in the same order as the fields in the struct. The possible values are as follows:

- `strict`: Elements/groups must be in order, and no other elements can be interspersed between them.
- `loose`: Elements/groups must be in order, but incorrect elements can be interspersed between them. This can include elements in parent groups/elements.
- `none`: Elements/groups can be in any order (default).

<!-- TODO: Some examples -->

This behaviour works on nested levels, so if you have an element with `children_order = "loose"`, and a group with `children_order = "none"` inside it, the elements listed in the group can be in any order, but they must be in order with respect to the other elements in the parent element.

<!-- TODO: Some examples -->

## `Serialize`/`Deserialize` - Nothing on enums

All variants must have exactly one unnamed field, and the type of that field must implement `Serialize`/`Deserialize`.

When serializing, it will try to serialize each type in order of the variants. If it succeeds, it will wrap the serialized value in the variant and return it.

When deserializing, it deserializes the field and then wraps it in the variant.
