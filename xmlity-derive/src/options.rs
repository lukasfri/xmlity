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
    Loose,
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
            "lowercase" => Ok(RenameRule::LowerCase),
            "UPPERCASE" => Ok(RenameRule::UpperCase),
            "PascalCase" => Ok(RenameRule::PascalCase),
            "camelCase" => Ok(RenameRule::CamelCase),
            "snake_case" => Ok(RenameRule::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Ok(RenameRule::ScreamingSnakeCase),
            "kebab-case" => Ok(RenameRule::KebabCase),
            "SCREAMING-KEBAB-CASE" => Ok(RenameRule::ScreamingKebabCase),
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
            "iterator" => Ok(Extendable::Iterator),
            "single" => Ok(Extendable::Single),
            "none" => Ok(Extendable::None),
            _ => Err(darling::Error::unknown_value(value)),
        }
    }

    fn from_bool(value: bool) -> darling::Result<Self> {
        if value {
            Ok(Extendable::Single)
        } else {
            Ok(Extendable::None)
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
        use syn::Attribute;

        use crate::common::Prefix;

        use super::*;

        #[derive(FromAttributes)]
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
            pub allow_unknown_children: bool,
            #[darling(default)]
            /// Deserialize only
            pub allow_unknown_attributes: bool,
            #[darling(default)]
            /// Deserialize only
            pub deserialize_any_name: bool,
            #[darling(default)]
            /// Deserialize only
            pub attribute_order: ElementOrder,
            #[darling(default)]
            /// Deserialize only
            pub children_order: ElementOrder,
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

        #[derive(FromAttributes)]
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

        #[derive(FromAttributes)]
        #[darling(attributes(xvalue))]
        pub struct RootValueOpts {
            pub value: Option<String>,
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
        use crate::common::Prefix;

        use super::*;

        #[derive(FromAttributes, Clone)]
        #[darling(attributes(xelement))]
        pub struct ElementOpts {
            #[darling(default)]
            pub default: bool,
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
            pub extendable: Extendable,
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
            pub fn should_unwrap_default(&self) -> bool {
                match self {
                    ChildOpts::Value(ValueOpts { default, .. }) => *default,
                    ChildOpts::Element(ElementOpts { default, .. }) => *default,
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
        }

        #[derive(Clone)]
        pub struct AttributeDeclaredOpts {
            pub default: bool,
            pub name: Option<LocalName<'static>>,
            pub namespace: Option<XmlNamespace<'static>>,
            pub namespace_expr: Option<Expr>,
            pub preferred_prefix: Option<Prefix<'static>>,
            pub enforce_prefix: bool,
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
            pub fn should_unwrap_default(&self) -> bool {
                match self {
                    AttributeOpts::Deferred(AttributeDeferredOpts { default }) => *default,
                    AttributeOpts::Declared(AttributeDeclaredOpts { default, .. }) => *default,
                }
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
                }

                let raw = FieldAttributeRawOpts::from_attributes(&[attribute])
                    .map(Some)
                    .map_err(DeriveError::Darling)?;

                let Some(raw) = raw else {
                    return Ok(None);
                };

                if raw.deferred {
                    if raw.name.is_some() {
                        return Err(DeriveError::custom(
                            "name can not be set if deferred is set",
                        ));
                    }

                    if raw.namespace.is_some() {
                        return Err(DeriveError::custom(
                            "namespace can not be set if deferred is set",
                        ));
                    }

                    if raw.namespace_expr.is_some() {
                        return Err(DeriveError::custom(
                            "namespace_expr can not be set if deferred is set",
                        ));
                    }

                    if raw.preferred_prefix.is_some() {
                        return Err(DeriveError::custom(
                            "preferred_prefix can not be set if deferred is set",
                        ));
                    }

                    if raw.enforce_prefix.is_some() {
                        return Err(DeriveError::custom(
                            "enforce_prefix can not be set if deferred is set",
                        ));
                    }

                    Ok(Some(Self::Deferred(AttributeDeferredOpts {
                        default: raw.default,
                    })))
                } else {
                    Ok(Some(Self::Declared(AttributeDeclaredOpts {
                        default: raw.default,
                        name: raw.name,
                        namespace: raw.namespace,
                        namespace_expr: raw.namespace_expr,
                        preferred_prefix: raw.preferred_prefix,
                        enforce_prefix: raw.enforce_prefix.unwrap_or(false),
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
        use super::*;

        #[derive(FromAttributes, Default)]
        #[darling(attributes(xvalue))]
        pub struct RootValueOpts {
            #[darling(default)]
            pub rename_all: RenameRule,
            #[darling(default)]
            #[allow(dead_code)]
            pub serialization_format: TextSerializationFormat,
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
        }

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
