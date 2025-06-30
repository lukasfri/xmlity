#![allow(dead_code)]
use std::borrow::Cow;

use darling::{FromAttributes, FromMeta};
use syn::{DeriveInput, Expr};

use crate::{
    common::{ExpandedName, LocalName, XmlNamespace},
    DeriveError,
};

#[derive(Debug, Clone, Copy, Default, FromMeta, PartialEq)]
#[darling(rename_all = "snake_case")]
pub enum GroupOrder {
    Strict,
    Loose,
    #[default]
    None,
}

#[derive(Debug, Clone, Copy, Default, FromMeta, PartialEq)]
#[darling(rename_all = "snake_case")]
pub enum ElementOrder {
    Strict,
    #[default]
    None,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum RenameRule {
    LowerCase,
    UpperCase,
    #[default]
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

impl RenameRule {
    /// Apply a renaming rule to an enum variant, returning the version expected in the source.
    pub fn apply_to_variant(self, variant: &str) -> String {
        use self::RenameRule::*;
        match self {
            PascalCase => variant.to_owned(),
            LowerCase => variant.to_ascii_lowercase(),
            UpperCase => variant.to_ascii_uppercase(),
            CamelCase => variant[..1].to_ascii_lowercase() + &variant[1..],
            SnakeCase => {
                let mut snake = String::new();
                for (i, ch) in variant.char_indices() {
                    if i > 0 && ch.is_uppercase() {
                        snake.push('_');
                    }
                    snake.push(ch.to_ascii_lowercase());
                }
                snake
            }
            ScreamingSnakeCase => SnakeCase.apply_to_variant(variant).to_ascii_uppercase(),
            KebabCase => SnakeCase.apply_to_variant(variant).replace('_', "-"),
            ScreamingKebabCase => ScreamingSnakeCase
                .apply_to_variant(variant)
                .replace('_', "-"),
        }
    }
}

impl FromMeta for RenameRule {
    fn from_string(value: &str) -> darling::Result<Self> {
        match value {
            "lowercase" => Ok(Self::LowerCase),
            "UPPERCASE" => Ok(Self::UpperCase),
            "PascalCase" => Ok(Self::PascalCase),
            "camelCase" => Ok(Self::CamelCase),
            "snake_case" => Ok(Self::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Ok(Self::ScreamingSnakeCase),
            "kebab-case" => Ok(Self::KebabCase),
            "SCREAMING-KEBAB-CASE" => Ok(Self::ScreamingKebabCase),
            _ => Err(darling::Error::unknown_value(value)),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum AllowUnknown {
    Any,
    #[default]
    AtEnd,
    None,
}

impl FromMeta for AllowUnknown {
    fn from_string(value: &str) -> darling::Result<Self> {
        match value {
            "any" => Ok(Self::Any),
            "at_end" => Ok(Self::AtEnd),
            "none" => Ok(Self::None),
            _ => Err(darling::Error::unknown_value(value)),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Extendable {
    Iterator,
    Single,
    #[default]
    None,
}

impl FromMeta for Extendable {
    fn from_string(value: &str) -> darling::Result<Self> {
        match value {
            "iterator" => Ok(Self::Iterator),
            "single" => Ok(Self::Single),
            "none" => Ok(Self::None),
            _ => Err(darling::Error::unknown_value(value)),
        }
    }

    fn from_bool(value: bool) -> darling::Result<Self> {
        if value {
            Ok(Self::Single)
        } else {
            Ok(Self::None)
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum IgnoreWhitespace {
    #[default]
    Any,
    None,
}

impl FromMeta for IgnoreWhitespace {
    fn from_string(value: &str) -> darling::Result<Self> {
        match value {
            "any" => Ok(Self::Any),
            "none" => Ok(Self::None),
            _ => Err(darling::Error::unknown_value(value)),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, FromMeta, PartialEq)]
pub enum TextSerializationFormat {
    CData,
    #[default]
    Text,
}

pub trait WithExpandedName {
    fn name(&self) -> Option<LocalName<'_>>;
    fn namespace(&self) -> Option<XmlNamespace<'_>>;
    fn namespace_expr(&self) -> Option<Expr>;
}

pub trait WithExpandedNameExt: WithExpandedName {
    fn expanded_name<'a>(&'a self, ident: &'a str) -> ExpandedName<'a>;
}

impl<T: WithExpandedName> WithExpandedNameExt for T {
    fn expanded_name<'a>(&'a self, default_local_name: &'a str) -> ExpandedName<'a> {
        if self.namespace().is_some() {
            ExpandedName::new(
                self.name()
                    .unwrap_or(LocalName(Cow::Borrowed(default_local_name))),
                self.namespace(),
            )
        } else {
            ExpandedName::new_ref(
                self.name()
                    .unwrap_or(LocalName(Cow::Borrowed(default_local_name))),
                self.namespace_expr(),
            )
        }
    }
}

/// Options for both structs and enum variants
pub mod records {
    use super::*;

    pub mod roots {
        use syn::{parse_quote, Attribute, Path};

        use crate::common::Prefix;

        use super::*;

        #[derive(FromAttributes, Clone)]
        #[darling(attributes(xelement))]
        pub struct RootElementOpts {
            #[darling(default)]
            pub name: Option<LocalName<'static>>,
            #[darling(default)]
            pub namespace: Option<XmlNamespace<'static>>,
            #[darling(default)]
            pub namespace_expr: Option<Expr>,
            #[darling(default)]
            /// Serialize only
            pub preferred_prefix: Option<Prefix<'static>>,
            #[darling(default)]
            /// Serialize only
            pub enforce_prefix: bool,
            #[darling(default)]
            /// Deserialize only
            pub allow_unknown_children: AllowUnknown,
            #[darling(default)]
            /// Deserialize only
            pub allow_unknown_attributes: AllowUnknown,
            #[darling(default)]
            /// Deserialize only
            pub deserialize_any_name: bool,
            #[darling(default)]
            /// Deserialize only
            pub attribute_order: ElementOrder,
            #[darling(default)]
            /// Deserialize only
            pub children_order: ElementOrder,
            #[darling(default)]
            /// Deserialize only
            pub ignore_whitespace: IgnoreWhitespace,
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
            #[darling(default)]
            pub name: Option<LocalName<'static>>,
            #[darling(default)]
            pub namespace: Option<XmlNamespace<'static>>,
            #[darling(default)]
            pub namespace_expr: Option<Expr>,
            #[darling(default)]
            /// Serialize only
            pub preferred_prefix: Option<Prefix<'static>>,
            #[darling(default)]
            /// Serialize only
            pub enforce_prefix: bool,
            #[darling(default)]
            /// Deserialize only
            pub deserialize_any_name: bool,
        }

        impl RootAttributeOpts {
            pub fn parse(attrs: &[Attribute]) -> Result<Option<Self>, DeriveError> {
                let Some(attr) = attrs.iter().find(|attr| attr.path().is_ident("xattribute"))
                else {
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
            pub value: Option<String>,
            #[darling(default)]
            /// Deserialize only
            pub ignore_whitespace: IgnoreWhitespace,
            #[darling(default)]
            /// Deserialize only
            pub allow_unknown: AllowUnknown,
            #[darling(default)]
            pub order: ElementOrder,
            #[darling(default)]
            pub with: Option<Path>,
            #[darling(default)]
            pub serialize_with: Option<Expr>,
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
            /// Deserialize only
            pub attribute_order: GroupOrder,
            #[darling(default)]
            /// Deserialize only
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
            #[darling(default)]
            pub default: bool,
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
            #[darling(default)]
            pub default: bool,
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
            pub default: bool,
            pub default_with: Option<Path>,
            pub skip_serializing_if: Option<Path>,
            pub optional: bool,
        }

        #[derive(Clone)]
        pub struct AttributeDeclaredOpts {
            pub default: bool,
            pub default_with: Option<Path>,
            pub name: Option<LocalName<'static>>,
            pub namespace: Option<XmlNamespace<'static>>,
            pub namespace_expr: Option<Expr>,
            pub preferred_prefix: Option<Prefix<'static>>,
            pub enforce_prefix: bool,
            pub skip_serializing_if: Option<Path>,
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
                    FieldOpts::Group(group_opts) => {
                        Some(FieldAttributeGroupOpts::Group(group_opts))
                    }
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
}

pub mod enums {
    use super::*;

    pub mod roots {
        use syn::{parse_quote, Path};

        use super::*;

        #[derive(FromAttributes, Default)]
        #[darling(attributes(xvalue))]
        pub struct RootValueOpts {
            #[darling(default)]
            pub rename_all: RenameRule,
            #[darling(default)]
            #[allow(dead_code)]
            pub serialization_format: TextSerializationFormat,
            #[darling(default)]
            pub with: Option<Path>,
            #[darling(default)]
            pub serialize_with: Option<Expr>,
            #[darling(default)]
            pub deserialize_with: Option<Expr>,
        }

        impl RootValueOpts {
            pub fn parse(ast: &DeriveInput) -> Result<Option<Self>, DeriveError> {
                let Some(attr) = ast.attrs.iter().find(|attr| attr.path().is_ident("xvalue"))
                else {
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

        #[allow(clippy::large_enum_variant)]
        pub enum RootOpts {
            None,
            Value(RootValueOpts),
        }

        impl RootOpts {
            pub fn parse(ast: &syn::DeriveInput) -> Result<Self, DeriveError> {
                let value_opts = RootValueOpts::parse(ast)?;

                match value_opts {
                    Some(value_opts) => Ok(RootOpts::Value(value_opts)),
                    None => Ok(RootOpts::None),
                }
            }
        }
    }

    pub mod variants {
        use syn::Attribute;

        use crate::options::records::roots::{RootAttributeOpts, RootElementOpts};

        use super::*;

        #[derive(Default, FromAttributes, Clone)]
        #[darling(attributes(xvalue))]
        pub struct RootValueOpts {
            pub value: Option<String>,
            #[darling(default)]
            /// Deserialize only
            pub ignore_whitespace: IgnoreWhitespace,
            #[darling(default)]
            /// Deserialize only
            pub allow_unknown: AllowUnknown,
            #[darling(default)]
            pub order: ElementOrder,
        }

        impl RootValueOpts {
            pub fn parse(attrs: &[Attribute]) -> Result<Option<Self>, DeriveError> {
                let Some(attr) = attrs.iter().find(|attr| attr.path().is_ident("xvalue")) else {
                    return Ok(None);
                };

                let opts = Self::from_attributes(&[attr.clone()])?;
                Ok(Some(opts))
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
}

#[derive(Clone)]
pub struct FieldWithOpts<I, Opts> {
    // If the field is indexed, this is none.
    pub field_ident: I,
    pub field_type: syn::Type,
    pub options: Opts,
}

impl<A, T> FieldWithOpts<A, T> {
    pub fn map_options<U, F: FnOnce(T) -> U>(self, f: F) -> FieldWithOpts<A, U> {
        FieldWithOpts {
            field_ident: self.field_ident,
            field_type: self.field_type,
            options: f(self.options),
        }
    }

    pub fn map_options_opt<U, F: FnOnce(T) -> Option<U>>(
        self,
        f: F,
    ) -> Option<FieldWithOpts<A, U>> {
        f(self.options).map(|options| FieldWithOpts {
            field_ident: self.field_ident,
            field_type: self.field_type,
            options,
        })
    }

    pub fn map_ident<U, F: FnOnce(A) -> U>(self, f: F) -> FieldWithOpts<U, T> {
        FieldWithOpts {
            field_ident: f(self.field_ident),
            field_type: self.field_type,
            options: (self.options),
        }
    }
}
