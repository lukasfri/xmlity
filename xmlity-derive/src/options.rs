#![allow(dead_code)]
use std::borrow::Cow;

use darling::{FromAttributes, FromMeta};
use quote::ToTokens;
use syn::{DeriveInput, Expr};

use crate::{DeriveError, ExpandedName, FieldIdent};

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
    fn expanded_name<'a>(&'a self, ident: &'a str) -> ExpandedName<'a> {
        if self.namespace().is_some() {
            ExpandedName::new(
                self.name().unwrap_or(LocalName(Cow::Borrowed(ident))),
                self.namespace(),
            )
        } else {
            ExpandedName::new_ref(
                self.name().unwrap_or(LocalName(Cow::Borrowed(ident))),
                self.namespace_expr(),
            )
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocalName<'a>(pub Cow<'a, str>);

impl<'a> LocalName<'a> {
    pub fn as_ref<'b: 'a>(&'b self) -> LocalName<'b> {
        Self(Cow::Borrowed(&self.0))
    }
}

impl FromMeta for LocalName<'_> {
    fn from_string(value: &str) -> darling::Result<Self> {
        // TODO: Validate local name
        Ok(LocalName(Cow::Owned(value.to_owned())))
    }
}

impl ToTokens for LocalName<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.0;
        tokens.extend(quote::quote! { <::xmlity::LocalName as ::core::str::FromStr>::from_str(#name).expect("XML name in derive macro is invalid. This is a bug in xmlity. Please report it.") })
    }
}

#[derive(Debug, Clone)]
pub struct XmlNamespace<'a>(pub Cow<'a, str>);

impl<'a> XmlNamespace<'a> {
    pub fn as_ref<'b: 'a>(&'b self) -> XmlNamespace<'b> {
        Self(Cow::Borrowed(&self.0))
    }
}

impl FromMeta for XmlNamespace<'_> {
    fn from_string(value: &str) -> darling::Result<Self> {
        // TODO: Validate namespace
        Ok(XmlNamespace(Cow::Owned(value.to_owned())))
    }
}

impl ToTokens for XmlNamespace<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let namespace = &self.0;
        tokens.extend(quote::quote! { <::xmlity::XmlNamespace as ::core::str::FromStr>::from_str(#namespace).expect("XML namespace in derive macro is invalid. This is a bug in xmlity. Please report it.") })
    }
}

#[derive(Debug, Default, Clone)]
pub struct Prefix<'a>(pub Cow<'a, str>);

impl<'a> Prefix<'a> {
    pub fn as_ref<'b: 'a>(&'b self) -> Prefix<'b> {
        Self(Cow::Borrowed(&self.0))
    }
}

impl FromMeta for Prefix<'_> {
    fn from_string(value: &str) -> darling::Result<Self> {
        // TODO: Validate prefix
        Ok(Prefix(Cow::Owned(value.to_owned())))
    }
}

impl ToTokens for Prefix<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let prefix = &self.0;
        tokens.extend(quote::quote! { ::xmlity::Prefix::new(#prefix).expect("XML prefix in derive macro is invalid. This is a bug in xmlity. Please report it.") })
    }
}

#[derive(FromAttributes)]
#[darling(attributes(xelement))]
pub struct XmlityRootElementDeriveOpts {
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

impl XmlityRootElementDeriveOpts {
    pub fn parse(ast: &DeriveInput) -> Result<Option<Self>, DeriveError> {
        let Some(attr) = ast
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("xelement"))
        else {
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

impl WithExpandedName for XmlityRootElementDeriveOpts {
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
pub struct XmlityRootAttributeDeriveOpts {
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

impl XmlityRootAttributeDeriveOpts {
    pub fn parse(ast: &DeriveInput) -> Result<Option<Self>, DeriveError> {
        let Some(attr) = ast
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("xattribute"))
        else {
            return Ok(None);
        };

        let opts = Self::from_attributes(&[attr.clone()])?;
        Ok(Some(opts))
    }
}

impl WithExpandedName for XmlityRootAttributeDeriveOpts {
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

#[derive(FromAttributes, Default)]
#[darling(attributes(xgroup))]
pub struct XmlityRootGroupDeriveOpts {
    #[darling(default)]
    /// Deserialize only
    pub attribute_order: GroupOrder,
    #[darling(default)]
    /// Deserialize only
    pub children_order: GroupOrder,
}

impl XmlityRootGroupDeriveOpts {
    pub fn parse(ast: &DeriveInput) -> Result<Option<Self>, DeriveError> {
        let Some(attr) = ast.attrs.iter().find(|attr| attr.path().is_ident("xgroup")) else {
            return Ok(None);
        };

        let opts = Self::from_attributes(&[attr.clone()])?;
        Ok(Some(opts))
    }
}

#[derive(FromAttributes, Default)]
#[darling(attributes(xvalue))]
pub struct XmlityRootValueDeriveOpts {
    #[darling(default)]
    pub rename_all: RenameRule,
    #[darling(default)]
    #[allow(dead_code)]
    pub serialization_format: TextSerializationFormat,
}

impl XmlityRootValueDeriveOpts {
    pub fn parse(ast: &DeriveInput) -> Result<Option<Self>, DeriveError> {
        let Some(attr) = ast.attrs.iter().find(|attr| attr.path().is_ident("xvalue")) else {
            return Ok(None);
        };

        let opts = Self::from_attributes(&[attr.clone()])?;
        Ok(Some(opts))
    }
}

#[derive(FromAttributes, Default, Clone)]
#[darling(attributes(xvalue))]
pub struct XmlityFieldValueDeriveOpts {
    #[darling(default)]
    pub default: bool,
    #[darling(default)]
    pub extendable: bool,
}

impl XmlityFieldValueDeriveOpts {
    pub fn from_field(field: &syn::Field) -> Result<Option<Self>, DeriveError> {
        let Some(attribute) = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("xvalue"))
            .cloned()
        else {
            return Ok(None);
        };
        Self::from_attributes(&[attribute])
            .map(Some)
            .map_err(DeriveError::Darling)
    }
}

#[derive(FromAttributes, Clone)]
#[darling(attributes(xattribute))]
pub struct XmlityFieldAttributeDeriveOpts {
    #[darling(default)]
    pub default: bool,
}

impl XmlityFieldAttributeDeriveOpts {
    pub fn from_field(field: &syn::Field) -> Result<Option<Self>, DeriveError> {
        let Some(attribute) = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("xattribute"))
            .cloned()
        else {
            return Ok(None);
        };
        Self::from_attributes(&[attribute])
            .map(Some)
            .map_err(DeriveError::Darling)
    }
}

#[derive(FromAttributes, Clone)]
#[darling(attributes(xgroup))]
pub struct XmlityFieldGroupDeriveOpts {}

impl XmlityFieldGroupDeriveOpts {
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
pub enum XmlityFieldDeriveOpts {
    Value(XmlityFieldValueDeriveOpts),
    Attribute(XmlityFieldAttributeDeriveOpts),
    Group(XmlityFieldGroupDeriveOpts),
}

#[derive(Clone)]
pub enum XmlityFieldAttributeGroupDeriveOpts {
    Attribute(XmlityFieldAttributeDeriveOpts),
    Group(XmlityFieldGroupDeriveOpts),
}

#[derive(Clone)]
pub enum XmlityFieldValueGroupDeriveOpts {
    Value(XmlityFieldValueDeriveOpts),
    Group(XmlityFieldGroupDeriveOpts),
}

impl XmlityFieldDeriveOpts {
    pub fn from_field(field: &syn::Field) -> Result<Self, DeriveError> {
        let element = XmlityFieldValueDeriveOpts::from_field(field)?;
        let attribute = XmlityFieldAttributeDeriveOpts::from_field(field)?;
        let group = XmlityFieldGroupDeriveOpts::from_field(field)?;
        Ok(match (element, attribute, group) {
            (Some(element), None, None) => Self::Value(element),
            (None, Some(attribute), None) => Self::Attribute(attribute),
            (None, None, Some(group)) => Self::Group(group),
            (None, None, None) => Self::Value(XmlityFieldValueDeriveOpts::default()),
            _ => {
                return Err(DeriveError::custom(
                    "Cannot have multiple xmlity field attributes on the same field.",
                ))
            }
        })
    }
}

#[derive(Clone)]
pub struct DeserializeField<BuilderFieldIdent, OptionType> {
    pub builder_field_ident: BuilderFieldIdent,
    // If the field is indexed, this is none.
    pub field_ident: FieldIdent,
    pub field_type: syn::Type,
    pub options: OptionType,
}

impl<A, T> DeserializeField<A, T> {
    pub fn map_options<U, F: FnOnce(T) -> U>(self, f: F) -> DeserializeField<A, U> {
        DeserializeField {
            builder_field_ident: self.builder_field_ident,
            field_ident: self.field_ident,
            field_type: self.field_type,
            options: f(self.options),
        }
    }

    pub fn map_options_opt<U, F: FnOnce(T) -> Option<U>>(
        self,
        f: F,
    ) -> Option<DeserializeField<A, U>> {
        f(self.options).map(|options| DeserializeField {
            builder_field_ident: self.builder_field_ident,
            field_ident: self.field_ident,
            field_type: self.field_type,
            options,
        })
    }
}

#[derive(Clone)]
pub struct SerializeField<OptionType> {
    // If the field is indexed, this is none.
    pub field_ident: FieldIdent,
    pub field_type: syn::Type,
    pub options: OptionType,
}

#[allow(dead_code)]
impl<T> SerializeField<T> {
    pub fn map_options<U, F: FnOnce(T) -> U>(self, f: F) -> SerializeField<U> {
        SerializeField {
            field_ident: self.field_ident,
            field_type: self.field_type,
            options: f(self.options),
        }
    }

    pub fn map_options_opt<U, F: FnOnce(T) -> Option<U>>(self, f: F) -> Option<SerializeField<U>> {
        f(self.options).map(|options| SerializeField {
            field_ident: self.field_ident,
            field_type: self.field_type,
            options,
        })
    }
}
