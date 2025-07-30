#![allow(dead_code)]
use std::borrow::Cow;

use darling::{FromAttributes, FromMeta};
use syn::{DeriveInput, Expr};

use crate::{
    common::{ExpandedName, LocalName, XmlNamespace},
    DeriveError,
};

pub mod enums;
pub mod records;

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
        match self {
            Self::PascalCase => variant.to_owned(),
            Self::LowerCase => variant.to_ascii_lowercase(),
            Self::UpperCase => variant.to_ascii_uppercase(),
            Self::CamelCase => variant[..1].to_ascii_lowercase() + &variant[1..],
            Self::SnakeCase => {
                let mut snake = String::new();
                for (i, ch) in variant.char_indices() {
                    if i > 0 && ch.is_uppercase() {
                        snake.push('_');
                    }
                    snake.push(ch.to_ascii_lowercase());
                }
                snake
            }
            Self::ScreamingSnakeCase => Self::SnakeCase
                .apply_to_variant(variant)
                .to_ascii_uppercase(),
            Self::KebabCase => Self::SnakeCase.apply_to_variant(variant).replace('_', "-"),
            Self::ScreamingKebabCase => Self::ScreamingSnakeCase
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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum IgnoreComments {
    #[default]
    Any,
    None,
}

impl FromMeta for IgnoreComments {
    fn from_string(value: &str) -> darling::Result<Self> {
        match value {
            "any" => Ok(Self::Any),
            "none" => Ok(Self::None),
            _ => Err(darling::Error::unknown_value(value)),
        }
    }
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
