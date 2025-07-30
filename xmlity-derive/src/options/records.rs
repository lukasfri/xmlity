//! Options for both structs and enum variants
use super::*;

pub mod roots {
    use syn::{parse_quote, Attribute, Path};

    use crate::common::Prefix;

    use super::*;

    #[derive(FromAttributes, Clone)]
    #[darling(attributes(xelement))]
    pub struct RootElementOpts {
        /// The name to serialize to and deserialize from.
        ///
        /// If not specified, the name of the struct or enum variant is used.
        #[darling(default)]
        pub name: Option<LocalName<'static>>,
        /// The namespace of the element, defined as a string.
        ///
        /// This is exclusive with [`namespace_expr`].
        ///
        /// If none of these are specified, the absence of a namespace is assumed.
        #[darling(default)]
        pub namespace: Option<XmlNamespace<'static>>,
        /// The namespace of the element given as an expression to an [`xmlity::XmlNamespace`] value.
        ///
        /// This is exclusive with [`namespace`].
        ///
        /// If none of these are specified, the absence of a namespace is assumed.
        #[darling(default)]
        pub namespace_expr: Option<Expr>,
        /// The element is serialized with the given prefix.
        ///
        /// *Serialize only*
        #[darling(default)]
        pub preferred_prefix: Option<Prefix<'static>>,
        /// Always set the prefix of the element to the prefix set in `preferred_prefix`.
        ///
        /// *Serialize only*
        #[darling(default)]
        pub enforce_prefix: bool,
        /// Allow unknown children when deserializing.
        /// - `Any`: Allow any unknown children.
        /// - `AtEnd` (*default*): Allow unknown children only at the end of the element.
        /// - `None`: Do not allow unknown children at all.
        ///
        /// *Deserialize only*
        #[darling(default)]
        pub allow_unknown_children: AllowUnknown,
        /// Allow unknown attributes when deserializing.
        /// - `Any`: Allow any unknown attributes.
        /// - `AtEnd` (*default*): Allow unknown attributes only at the end of the element.
        /// - `None`: Do not allow unknown attributes at all.
        ///
        /// Default is `AtEnd`.
        ///
        /// *Deserialize only*
        #[darling(default)]
        pub allow_unknown_attributes: AllowUnknown,
        /// Allow any name for the element when deserializing.
        ///
        /// *Deserialize only*
        #[darling(default)]
        pub deserialize_any_name: bool,
        /// Set if the order of attributes is important when serializing or deserializing.
        /// - `Strict`: The order of attributes must match the order in the struct or enum variant.
        /// - `None` (*default*): The order of attributes does not matter, but the attributes must be present.
        ///
        /// *Deserialize only*
        #[darling(default)]
        pub attribute_order: ElementOrder,
        /// Set if the order of children is important when serializing or deserializing.
        /// - `Strict`: The order of children must match the order in the struct or enum variant.
        /// - `None` (*default*): The order of children does not matter, but the children must be present.
        ///
        /// *Deserialize only*
        #[darling(default)]
        pub children_order: ElementOrder,
        /// Set if whitespace should be ignored when deserializing.
        /// - `Any` (*default*): Ignore any whitespace.
        /// - `None`: Do not ignore whitespace.
        ///
        /// *Deserialize only*
        #[darling(default)]
        pub ignore_whitespace: IgnoreWhitespace,
        /// Set if comments should be ignored when deserializing.
        /// - `Any` (*default*): Ignore any comments.
        /// - `None`: Do not ignore comments.
        ///
        /// *Deserialize only*
        #[darling(default)]
        pub ignore_comments: IgnoreComments,
    }

    impl RootElementOpts {
        pub fn parse(attrs: &[Attribute]) -> Result<Option<Self>, DeriveError> {
            let Some(attr) = attrs.iter().find(|attr| attr.path().is_ident("xelement")) else {
                return Ok(None);
            };

            let opts = Self::from_attributes(&[attr.clone()])?;
            if opts.namespace_expr.is_some() && opts.namespace.is_some() {
                return Err(DeriveError::custom(
                    "Cannot specify both `namespace` and `namespace_expr`",
                ));
            }
            Ok(Some(opts))
        }
    }

    impl WithExpandedName for RootElementOpts {
        fn name(&self) -> Option<LocalName<'_>> {
            self.name.clone()
        }

        fn namespace(&self) -> Option<XmlNamespace<'_>> {
            self.namespace.clone()
        }

        fn namespace_expr(&self) -> Option<Expr> {
            self.namespace_expr.clone()
        }
    }

    #[derive(FromAttributes, Clone)]
    #[darling(attributes(xattribute))]
    pub struct RootAttributeOpts {
        /// The name to serialize to and deserialize from.
        ///
        /// If not specified, the name of the struct is used.
        #[darling(default)]
        pub name: Option<LocalName<'static>>,
        /// The namespace of the attribute, defined as a string.
        ///
        /// This is exclusive with [`namespace_expr`].
        ///
        /// If none of these are specified, the absence of a namespace is assumed.
        #[darling(default)]
        pub namespace: Option<XmlNamespace<'static>>,
        /// The namespace of the attribute given as an expression to an [`xmlity::XmlNamespace`] value.
        ///
        /// This is exclusive with [`namespace`].
        ///
        /// If none of these are specified, the absence of a namespace is assumed.
        #[darling(default)]
        pub namespace_expr: Option<Expr>,
        /// The preferred prefix for the attribute, defined as a string.
        ///
        /// This is exclusive with [`enforce_prefix`].
        ///
        /// If none of these are specified, the absence of a prefix is assumed.
        ///
        /// *Serialize only*
        #[darling(default)]
        pub preferred_prefix: Option<Prefix<'static>>,
        /// Always set the prefix of the attribute to the prefix set in `preferred_prefix`.
        ///
        /// *Serialize only*
        #[darling(default)]
        pub enforce_prefix: bool,
        /// Always set the prefix of the attribute to the prefix set in `preferred_prefix`.
        ///
        /// *Deserialize only*
        #[darling(default)]
        pub deserialize_any_name: bool,
    }

    impl RootAttributeOpts {
        pub fn parse(attrs: &[Attribute]) -> Result<Option<Self>, DeriveError> {
            let Some(attr) = attrs.iter().find(|attr| attr.path().is_ident("xattribute")) else {
                return Ok(None);
            };

            let opts = Self::from_attributes(&[attr.clone()])?;
            Ok(Some(opts))
        }
    }

    impl WithExpandedName for RootAttributeOpts {
        fn name(&self) -> Option<LocalName<'_>> {
            self.name.clone()
        }

        fn namespace(&self) -> Option<XmlNamespace<'_>> {
            self.namespace.clone()
        }

        fn namespace_expr(&self) -> Option<Expr> {
            self.namespace_expr.clone()
        }
    }

    #[derive(Default, FromAttributes)]
    #[darling(attributes(xvalue))]
    pub struct RootValueOpts {
        /// The text value to serialize to and deserialize from.
        pub value: Option<String>,
        #[darling(default)]
        /// Set if whitespace should be ignored when deserializing.
        /// - `Any` (*default*): Ignore any whitespace.
        /// - `None`: Do not ignore whitespace.
        ///
        /// *Deserialize only*
        pub ignore_whitespace: IgnoreWhitespace,
        #[darling(default)]
        /// Set if comments should be ignored when deserializing.
        /// - `Any` (*default*): Ignore any comments.
        /// - `None`: Do not ignore comments.
        ///
        /// *Deserialize only*
        pub ignore_comments: IgnoreComments,
        /// Allow unknown values when deserializing.
        /// - `Any`: Allow any unknown values.
        /// - `AtEnd` (*default*): Allow unknown values only at the end of the element.
        /// - `None`: Do not allow unknown values at all.
        ///
        /// *Deserialize only*
        #[darling(default)]
        pub allow_unknown: AllowUnknown,
        /// Set if the order of values is important when serializing or deserializing.
        /// - `Strict`: The order of values must match the order in the struct or enum variant.
        /// - `None` (*default*): The order of values does not matter, but the values must be present.
        ///
        /// *Deserialize only*
        #[darling(default)]
        pub order: ElementOrder,
        /// The path to the module that provides the serialization and deserialization functions.
        ///
        /// `::serialize` and `::deserialize` will be appended to this path and used as the `serialize_with` and `deserialize_with` functions.
        #[darling(default)]
        pub with: Option<Path>,
        /// Use function to serialize the value.
        ///
        /// Should have signature like `pub fn serialize<S: xmlity::Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error>`
        #[darling(default)]
        pub serialize_with: Option<Expr>,
        /// Use function to deserialize the value.
        ///
        /// Should have signature like `fn deserialize<'de, D: xmlity::Deserializer<'de>>(deserializer: D) -> Result<T, D::Error>`
        #[darling(default)]
        pub deserialize_with: Option<Expr>,
    }

    impl RootValueOpts {
        pub fn parse(attrs: &[Attribute]) -> Result<Option<Self>, DeriveError> {
            let Some(attr) = attrs.iter().find(|attr| attr.path().is_ident("xvalue")) else {
                return Ok(None);
            };

            let opts = Self::from_attributes(&[attr.clone()])?;
            Ok(Some(opts))
        }

        pub fn serialize_with(&self) -> Option<Expr> {
            self.serialize_with
                .as_ref()
                .map(|serialize_with| {
                    parse_quote! {
                        #serialize_with
                    }
                })
                .or_else(|| {
                    self.with.as_ref().map(|with| {
                        parse_quote! {
                            #with::serialize
                        }
                    })
                })
        }

        pub fn deserialize_with(&self) -> Option<Expr> {
            self.deserialize_with
                .as_ref()
                .map(|deserialize_with| {
                    parse_quote! {
                        #deserialize_with
                    }
                })
                .or_else(|| {
                    self.with.as_ref().map(|with| {
                        parse_quote! {
                            #with::deserialize
                        }
                    })
                })
        }
    }

    #[derive(FromAttributes, Default)]
    #[darling(attributes(xgroup))]
    pub struct RootGroupOpts {
        #[darling(default)]
        /// Set if the order of attributes is important when serializing or deserializing.
        /// - `Strict`: The attributes must come directly after each other, and this group will try to deserialize them in one go.
        /// - `Loose`: The order of attributes must come relative to each other, but they can be separated by other attributes outside this group.
        /// - `None` (*default*): The order of attributes is not important.
        ///
        /// *Deserialize only*
        pub attribute_order: GroupOrder,
        #[darling(default)]
        /// Set if the order of children is important when serializing or deserializing.
        /// - `Strict`: The children must come directly after each other, and this group will try to deserialize them in one go.
        /// - `Loose`: The order of children must come relative to each other, but they can be separated by other children outside this group.
        /// - `None` (*default*): The order of children is not important.
        ///
        /// *Deserialize only*
        pub children_order: GroupOrder,
    }

    impl RootGroupOpts {
        pub fn parse(attrs: &[Attribute]) -> Result<Option<Self>, DeriveError> {
            let Some(attr) = attrs.iter().find(|attr| attr.path().is_ident("xgroup")) else {
                return Ok(None);
            };

            let opts = Self::from_attributes(&[attr.clone()])?;
            Ok(Some(opts))
        }
    }

    #[allow(clippy::large_enum_variant)]
    pub enum SerializeRootOpts {
        None,
        Element(RootElementOpts),
        Value(RootValueOpts),
    }

    impl SerializeRootOpts {
        pub fn parse(attrs: &[Attribute]) -> Result<Self, DeriveError> {
            let element_opts = RootElementOpts::parse(attrs)?;
            let value_opts = RootValueOpts::parse(attrs)?;

            match (element_opts, value_opts) {
                    (Some(element_opts), None) => Ok(Self::Element(element_opts)),
                    (None, Some(value_opts)) => Ok(Self::Value(value_opts)),
                    (None, None) => Ok(Self::None),
                    _ => Err(DeriveError::custom("Wrong options. Only one of `xelement`, `xattribute`, or `xvalue` can be used for root elements.")),
                }
        }
    }

    pub enum DeserializeRootOpts {
        None,
        Element(RootElementOpts),
        Attribute(RootAttributeOpts),
        Value(RootValueOpts),
    }

    impl DeserializeRootOpts {
        pub fn parse(attrs: &[Attribute]) -> Result<Self, DeriveError> {
            let element_opts = RootElementOpts::parse(attrs)?;
            let attribute_opts = RootAttributeOpts::parse(attrs)?;
            let value_opts = RootValueOpts::parse(attrs)?;

            match (element_opts, attribute_opts, value_opts) {
                    (Some(element_opts), None, None) => Ok(Self::Element(element_opts)),
                    (None, Some(attribute_opts), None) => Ok(Self::Attribute(attribute_opts)),
                    (None, None, Some(value_opts)) => Ok(Self::Value(value_opts)),
                    (None, None, None) => Ok(Self::None),
                    _ => Err(DeriveError::custom("Wrong options. Only one of `xelement`, `xattribute`, or `xvalue` can be used for root elements.")),
                }
        }
    }
}

pub mod fields {
    use quote::ToTokens;
    use syn::{parse_quote, Path};

    use crate::common::Prefix;

    use super::*;

    #[derive(FromAttributes, Clone)]
    #[darling(attributes(xelement))]
    pub struct ElementOpts {
        /// Default value for the field if the element is not present.
        #[darling(default)]
        pub default: bool,
        /// Use function to provide a default value for the field.
        ///
        /// Should have signature like `pub fn default_value() -> T`.
        #[darling(default)]
        pub default_with: Option<Path>,
        #[darling(default)]
        pub extendable: Extendable,
        #[darling(default)]
        pub name: Option<LocalName<'static>>,
        #[darling(default)]
        pub namespace: Option<XmlNamespace<'static>>,
        #[darling(default)]
        pub namespace_expr: Option<Expr>,
        #[darling(default)]
        pub preferred_prefix: Option<Prefix<'static>>,
        #[darling(default)]
        pub enforce_prefix: bool,
        #[darling(default)]
        pub optional: bool,
        #[darling(default)]
        pub group: bool,
        #[darling(default)]
        pub skip_serializing_if: Option<Path>,
    }

    impl ElementOpts {
        pub fn default_or_else(&self) -> Option<Expr> {
            if let Some(default_with) = self.default_with.as_ref() {
                Some(parse_quote! {
                    #default_with
                })
            } else if self.default || self.optional {
                Some(parse_quote! {
                    ::core::default::Default::default
                })
            } else {
                None
            }
        }

        pub fn skip_serializing_if<T: ToTokens>(&self, access: T) -> Option<Expr> {
            self.skip_serializing_if
                .as_ref()
                .map(|skip_serializing_if| {
                    parse_quote! {
                        #skip_serializing_if(#access)
                    }
                })
                .or(self.optional.then(|| {
                    parse_quote! {
                        ::core::option::Option::is_none(#access)
                    }
                }))
        }
    }

    impl WithExpandedName for ElementOpts {
        fn name(&self) -> Option<LocalName<'_>> {
            self.name.clone()
        }

        fn namespace(&self) -> Option<XmlNamespace<'_>> {
            self.namespace.clone()
        }

        fn namespace_expr(&self) -> Option<Expr> {
            self.namespace_expr.clone()
        }
    }

    #[derive(FromAttributes, Clone, Default)]
    #[darling(attributes(xvalue))]
    pub struct ValueOpts {
        /// Default value for the field if the element is not present.
        #[darling(default)]
        pub default: bool,
        /// Use function to provide a default value for the field.
        ///
        /// Should have signature like `pub fn default_value() -> T`.
        #[darling(default)]
        pub default_with: Option<Path>,
        #[darling(default)]
        pub extendable: Extendable,
        #[darling(default)]
        pub skip_serializing_if: Option<Path>,
    }

    impl ValueOpts {
        pub fn default_or_else(&self) -> Option<Expr> {
            if let Some(default_with) = &self.default_with {
                Some(parse_quote! {
                    #default_with
                })
            } else if self.default {
                Some(parse_quote! {
                    ::core::default::Default::default
                })
            } else {
                None
            }
        }

        pub fn skip_serializing_if<T: ToTokens>(&self, access: T) -> Option<Expr> {
            self.skip_serializing_if
                .as_ref()
                .map(|skip_serializing_if| {
                    parse_quote! {
                        #skip_serializing_if(#access)
                    }
                })
        }
    }

    #[allow(clippy::large_enum_variant)]
    #[derive(Clone)]
    pub enum ChildOpts {
        Value(ValueOpts),
        Element(ElementOpts),
    }

    impl Default for ChildOpts {
        fn default() -> Self {
            Self::Value(ValueOpts::default())
        }
    }

    impl ChildOpts {
        pub fn default_or_else(&self) -> Option<Expr> {
            let (default, default_with) = match self {
                ChildOpts::Value(ValueOpts {
                    default,
                    default_with,
                    ..
                }) => (*default, default_with),
                ChildOpts::Element(ElementOpts {
                    default,
                    default_with,
                    optional,
                    ..
                }) => (*default || *optional, default_with),
            };

            if let Some(default_with) = default_with {
                Some(parse_quote! {
                    #default_with
                })
            } else if default {
                Some(parse_quote! {
                    ::core::default::Default::default
                })
            } else {
                None
            }
        }

        pub fn from_field(field: &syn::Field) -> Result<Option<Self>, DeriveError> {
            let xvalue_attribute = field
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("xvalue"))
                .cloned();
            let xelement_attribute = field
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("xelement"))
                .cloned();

            match (xvalue_attribute, xelement_attribute) {
                (None, None) => Ok(None),
                (Some(_), Some(_)) => Err(DeriveError::custom(
                    "Cannot have both `xvalue` and `xelement` attributes on the same field.",
                )),
                (Some(xvalue_attribute), None) => Self::from_xvalue_attribute(xvalue_attribute),
                (None, Some(xelement_attribute)) => {
                    Self::from_xelement_attribute(xelement_attribute)
                }
            }
        }

        pub fn from_xvalue_attribute(
            xvalue_attribute: syn::Attribute,
        ) -> Result<Option<Self>, DeriveError> {
            let opts = ValueOpts::from_attributes(&[xvalue_attribute])?;
            Ok(Some(ChildOpts::Value(opts)))
        }

        pub fn from_xelement_attribute(
            xelement_attribute: syn::Attribute,
        ) -> Result<Option<Self>, DeriveError> {
            let opts = ElementOpts::from_attributes(&[xelement_attribute])?;
            Ok(Some(ChildOpts::Element(opts)))
        }
    }

    #[derive(Clone)]
    pub struct AttributeDeferredOpts {
        /// Default value for the field if the element is not present.
        pub default: bool,
        /// Use function to provide a default value for the field.
        ///
        /// Should have signature like `pub fn default_value() -> T`.
        pub default_with: Option<Path>,
        /// Use function to skip serializing the field if it is not set.
        ///
        /// Should have signature like `pub fn skip_serializing_if(value: &T) -> bool`.
        pub skip_serializing_if: Option<Path>,
        /// If the field is an [`Option<T>`], it will not be serialized if it is not set.
        pub optional: bool,
    }

    #[derive(Clone)]
    pub struct AttributeDeclaredOpts {
        /// Default value for the field if the element is not present.
        pub default: bool,
        /// Use function to provide a default value for the field.
        ///
        /// Should have signature like `pub fn default_value() -> T`.
        pub default_with: Option<Path>,
        /// The name to serialize to and deserialize from.
        ///
        /// If not specified, the name of the struct is used.
        pub name: Option<LocalName<'static>>,
        /// The namespace of the attribute, defined as a string.
        ///
        /// This is exclusive with [`namespace_expr`].
        ///
        /// If none of these are specified, the absence of a namespace is assumed.
        pub namespace: Option<XmlNamespace<'static>>,
        /// The namespace of the attribute given as an expression to an [`xmlity::XmlNamespace`] value.
        ///
        /// This is exclusive with [`namespace`].
        ///
        /// If none of these are specified, the absence of a namespace is assumed.
        pub namespace_expr: Option<Expr>,
        /// The preferred prefix for the attribute, defined as a string.
        ///
        /// This is exclusive with [`enforce_prefix`].
        ///
        /// If none of these are specified, the absence of a prefix is assumed.
        pub preferred_prefix: Option<Prefix<'static>>,
        /// Always set the prefix of the attribute to the prefix set in `preferred_prefix`.
        ///
        /// *Serialize only*
        pub enforce_prefix: bool,
        /// Use function to skip serializing the field if it is not set.
        ///
        /// Should have signature like `pub fn skip_serializing_if(value: &T) -> bool`.
        pub skip_serializing_if: Option<Path>,
        /// If the field is an [`Option<T>`], it will not be serialized if it is not set.
        pub optional: bool,
    }

    impl WithExpandedName for AttributeDeclaredOpts {
        fn name(&self) -> Option<LocalName<'_>> {
            self.name.clone()
        }

        fn namespace(&self) -> Option<XmlNamespace<'_>> {
            self.namespace.clone()
        }

        fn namespace_expr(&self) -> Option<Expr> {
            self.namespace_expr.clone()
        }
    }

    #[allow(clippy::large_enum_variant)]
    #[derive(Clone)]
    pub enum AttributeOpts {
        Deferred(AttributeDeferredOpts),
        Declared(AttributeDeclaredOpts),
    }

    impl AttributeOpts {
        pub fn default_or_else(&self) -> Option<Expr> {
            let (default, default_with, optional) = match self {
                AttributeOpts::Deferred(AttributeDeferredOpts {
                    default,
                    default_with,
                    optional,
                    ..
                }) => (default, default_with, optional),
                AttributeOpts::Declared(AttributeDeclaredOpts {
                    default,
                    default_with,
                    optional,
                    ..
                }) => (default, default_with, optional),
            };

            if let Some(default_with) = default_with {
                Some(parse_quote! {
                    #default_with
                })
            } else if *default || *optional {
                Some(parse_quote! {
                    ::core::default::Default::default
                })
            } else {
                None
            }
        }

        pub fn skip_serializing_if<T: ToTokens>(&self, access: T) -> Option<Expr> {
            let (skip_serializing_if, optional) = match self {
                AttributeOpts::Deferred(AttributeDeferredOpts {
                    skip_serializing_if,
                    optional,
                    ..
                }) => (skip_serializing_if, optional),
                AttributeOpts::Declared(AttributeDeclaredOpts {
                    skip_serializing_if,
                    optional,
                    ..
                }) => (skip_serializing_if, optional),
            };

            skip_serializing_if
                .as_ref()
                .map(|skip_serializing_if| {
                    parse_quote! {
                        #skip_serializing_if(#access)
                    }
                })
                .or(optional.then(|| {
                    parse_quote! {
                        ::core::option::Option::is_none(#access)
                    }
                }))
        }

        pub fn from_field(field: &syn::Field) -> Result<Option<Self>, DeriveError> {
            let Some(attribute) = field
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("xattribute"))
                .cloned()
            else {
                return Ok(None);
            };

            #[derive(FromAttributes)]
            #[darling(attributes(xattribute))]
            pub struct FieldAttributeRawOpts {
                #[darling(default)]
                pub default: bool,
                #[darling(default)]
                pub default_with: Option<Path>,
                #[darling(default)]
                pub deferred: bool,
                #[darling(default)]
                pub name: Option<LocalName<'static>>,
                #[darling(default)]
                pub namespace: Option<XmlNamespace<'static>>,
                #[darling(default)]
                pub namespace_expr: Option<Expr>,
                #[darling(default)]
                pub preferred_prefix: Option<Prefix<'static>>,
                #[darling(default)]
                pub enforce_prefix: Option<bool>,
                #[darling(default)]
                pub optional: bool,
                #[darling(default)]
                pub skip_serializing_if: Option<Path>,
            }

            let raw = FieldAttributeRawOpts::from_attributes(&[attribute])
                .map(Some)
                .map_err(DeriveError::Darling)?;

            let Some(raw) = raw else {
                return Ok(None);
            };

            if raw.deferred {
                let unallowed_fields = [
                    (raw.name.is_some(), "name"),
                    (raw.namespace.is_some(), "namespace"),
                    (raw.namespace_expr.is_some(), "namespace_expr"),
                    (raw.preferred_prefix.is_some(), "preferred_prefix"),
                    (raw.enforce_prefix.is_some(), "enforce_prefix"),
                ];
                if let Some((true, field)) =
                    unallowed_fields.iter().find(|(unallowed, _)| *unallowed)
                {
                    return Err(DeriveError::custom(format!(
                        "{field} can not be set if deferred is set"
                    )));
                }

                Ok(Some(Self::Deferred(AttributeDeferredOpts {
                    default: raw.default,
                    default_with: raw.default_with,
                    skip_serializing_if: raw.skip_serializing_if,
                    optional: raw.optional,
                })))
            } else {
                Ok(Some(Self::Declared(AttributeDeclaredOpts {
                    default: raw.default,
                    default_with: raw.default_with,
                    name: raw.name,
                    namespace: raw.namespace,
                    namespace_expr: raw.namespace_expr,
                    preferred_prefix: raw.preferred_prefix,
                    enforce_prefix: raw.enforce_prefix.unwrap_or(false),
                    skip_serializing_if: raw.skip_serializing_if,
                    optional: raw.optional,
                })))
            }
        }
    }

    #[derive(FromAttributes, Clone)]
    #[darling(attributes(xgroup))]
    pub struct GroupOpts {}

    impl GroupOpts {
        pub fn from_field(field: &syn::Field) -> Result<Option<Self>, DeriveError> {
            let Some(attribute) = field
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("xgroup"))
                .cloned()
            else {
                return Ok(None);
            };
            Self::from_attributes(&[attribute])
                .map(Some)
                .map_err(DeriveError::Darling)
        }
    }

    #[derive(Clone)]
    pub enum FieldOpts {
        Value(ChildOpts),
        Attribute(AttributeOpts),
        Group(GroupOpts),
    }

    impl FieldOpts {
        pub fn value_group(self) -> Option<FieldValueGroupOpts> {
            match self {
                FieldOpts::Value(child_opts) => Some(FieldValueGroupOpts::Value(child_opts)),
                FieldOpts::Attribute(_) => None,
                FieldOpts::Group(group_opts) => Some(FieldValueGroupOpts::Group(group_opts)),
            }
        }

        pub fn attribute(self) -> Option<AttributeOpts> {
            match self {
                FieldOpts::Value(_) => None,
                FieldOpts::Attribute(attribute_opts) => Some(attribute_opts),
                FieldOpts::Group(_) => None,
            }
        }

        pub fn attribute_group(self) -> Option<FieldAttributeGroupOpts> {
            match self {
                FieldOpts::Value(_) => None,
                FieldOpts::Attribute(attribute_opts) => {
                    Some(FieldAttributeGroupOpts::Attribute(attribute_opts))
                }
                FieldOpts::Group(group_opts) => Some(FieldAttributeGroupOpts::Group(group_opts)),
            }
        }
    }

    #[allow(clippy::large_enum_variant)]
    #[derive(Clone)]
    pub enum FieldAttributeGroupOpts {
        Attribute(AttributeOpts),
        Group(GroupOpts),
    }

    #[allow(clippy::large_enum_variant)]
    #[derive(Clone)]
    pub enum FieldValueGroupOpts {
        Value(ChildOpts),
        Group(GroupOpts),
    }

    impl FieldOpts {
        pub fn from_field(field: &syn::Field) -> Result<Self, DeriveError> {
            let element = ChildOpts::from_field(field)?;
            let attribute = AttributeOpts::from_field(field)?;
            let group = GroupOpts::from_field(field)?;
            Ok(match (element, attribute, group) {
                (Some(element), None, None) => Self::Value(element),
                (None, Some(attribute), None) => Self::Attribute(attribute),
                (None, None, Some(group)) => Self::Group(group),
                (None, None, None) => Self::Value(ChildOpts::default()),
                _ => {
                    return Err(DeriveError::custom(
                        "Cannot have multiple xmlity field attributes on the same field.",
                    ))
                }
            })
        }
    }
}
