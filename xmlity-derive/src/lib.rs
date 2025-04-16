//! # XMLity Derive
//!
//! This crate contains the proc-macros for XMLity, specifically the derive macros for [`Serialize`], [`SerializeAttribute`], [`Deserialize`], [`SerializationGroup`], and [`DeserializationGroup`].

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct _ReadMeDocTests;

use de::{DeriveDeserializationGroup, DeriveDeserialize};
use quote::{quote, ToTokens};
use ser::{DeriveSerializationGroup, DeriveSerialize, DeriveSerializeAttribute};
use syn::{parse_macro_input, DeriveInput, Expr};

mod de;
mod options;
use options::{LocalName, XmlNamespace};
mod ser;
mod utils;
use options::{DeserializeField, SerializeField};

enum DeriveError {
    Darling(darling::Error),
    Custom {
        message: String,
        span: proc_macro2::Span,
    },
}

type DeriveResult<T> = Result<T, DeriveError>;

impl From<darling::Error> for DeriveError {
    fn from(e: darling::Error) -> Self {
        DeriveError::Darling(e)
    }
}

impl From<syn::Error> for DeriveError {
    fn from(e: syn::Error) -> Self {
        DeriveError::Darling(e.into())
    }
}

impl DeriveError {
    fn into_compile_error(self) -> proc_macro2::TokenStream {
        match self {
            DeriveError::Darling(e) => e.write_errors(),
            DeriveError::Custom { message, span } => {
                syn::Error::new(span, message).to_compile_error()
            }
        }
    }

    fn custom<T: Into<String>>(error: T) -> Self {
        Self::custom_with_span(error, proc_macro2::Span::call_site())
    }

    fn custom_with_span<T: Into<String>, S: Into<proc_macro2::Span>>(error: T, span: S) -> Self {
        Self::Custom {
            message: error.into(),
            span: span.into(),
        }
    }
}

trait DeriveMacro {
    fn input_to_derive(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError>
    where
        Self: Sized;
}

trait DeriveMacroExt {
    fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream;
}

impl<T: DeriveMacro> DeriveMacroExt for T {
    fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
        let ast = parse_macro_input!(input as DeriveInput);
        T::input_to_derive(&ast)
            .unwrap_or_else(|e| e.into_compile_error())
            .into()
    }
}

/// Derives the [`xmlity::Serialize`] trait for a type.
#[proc_macro_derive(Serialize, attributes(xelement, xattribute, xgroup, xvalue))]
pub fn derive_serialize_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    DeriveSerialize::derive(item)
}

/// Derives the [`xmlity::SerializeAttribute`] trait for a type.
#[proc_macro_derive(SerializeAttribute, attributes(xattribute))]
pub fn derive_serialize_attribute_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    DeriveSerializeAttribute::derive(item)
}

/// Derives the [`xmlity::Deserialize`] trait for a type.
#[proc_macro_derive(Deserialize, attributes(xelement, xattribute, xgroup, xvalue))]
pub fn derive_deserialize_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    DeriveDeserialize::derive(item)
}

/// Derives the [`xmlity::SerializationGroup`] trait for a type.
#[proc_macro_derive(SerializationGroup, attributes(xelement, xattribute, xgroup))]
pub fn derive_serialization_group_attribute_fn(
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    DeriveSerializationGroup::derive(item)
}

/// Derives the [`xmlity::DeserializationGroup`] trait for a type.
#[proc_macro_derive(DeserializationGroup, attributes(xelement, xattribute, xgroup))]
pub fn derive_deserialization_group_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    DeriveDeserializationGroup::derive(item)
}

#[derive(Clone)]
enum FieldIdent {
    Named(syn::Ident),
    Indexed(syn::Index),
}

impl quote::ToTokens for FieldIdent {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FieldIdent::Named(ident) => ident.to_tokens(tokens),
            FieldIdent::Indexed(index) => index.to_tokens(tokens),
        }
    }
}
enum XmlNamespaceRef<'a> {
    Static(XmlNamespace<'a>),
    Dynamic(syn::Expr),
}

impl ToTokens for XmlNamespaceRef<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            XmlNamespaceRef::Static(namespace) => namespace.to_tokens(tokens),
            XmlNamespaceRef::Dynamic(expr) => expr.to_tokens(tokens),
        }
    }
}

struct ExpandedName<'a> {
    name: LocalName<'a>,
    namespace: Option<XmlNamespaceRef<'a>>,
}

impl<'a> ExpandedName<'a> {
    fn new(name: LocalName<'a>, namespace: Option<XmlNamespace<'a>>) -> Self {
        Self {
            name,
            namespace: namespace.map(XmlNamespaceRef::Static),
        }
    }
    fn new_ref(name: LocalName<'a>, namespace: Option<Expr>) -> Self {
        Self {
            name,
            namespace: namespace.map(XmlNamespaceRef::Dynamic),
        }
    }

    fn to_expression(Self { name, namespace }: &Self) -> proc_macro2::TokenStream {
        let xml_namespace = match namespace {
            Some(xml_namespace) => {
                quote! { ::core::option::Option::Some(#xml_namespace) }
            }
            None => quote! { ::core::option::Option::None },
        };

        quote! {
            ::xmlity::ExpandedName::new(#name, #xml_namespace)
        }
    }
}

impl quote::ToTokens for ExpandedName<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(Self::to_expression(self))
    }
}

fn non_bound_generics(generics: &syn::Generics) -> syn::Generics {
    let mut non_bound_generics = generics.to_owned();
    non_bound_generics.where_clause = None;
    non_bound_generics
        .lifetimes_mut()
        .for_each(|a| a.bounds.clear());
    non_bound_generics
        .type_params_mut()
        .for_each(|a| a.bounds.clear());

    non_bound_generics
}
