//! # XMLity Derive
//!
//! This crate contains the proc-macros for XMLity, specifically the derive macros for [`Serialize`], [`SerializeAttribute`], [`Deserialize`], [`SerializationGroup`], and [`DeserializationGroup`].
//!
//! Each of these macros has its own documentation, which can be found by following the links above.
//!
//! The attributes used by these macros are made to be compatible with their counterparts:
//! - [`Serialize`] and [`SerializeAttribute`] use the same attributes with the same options as [`Deserialize`].
//! - [`SerializationGroup`] use the same attributes with the same options as [`DeserializationGroup`].
//!
//! There are some attributes only used by either serialization or deserialization. These are highlighted in the documentation for each macro.
//!
//! ## Example
//! ```ignore
//! use xmlity_derive::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! #[xelement(name = "name")]
//! struct Name(String);
//!
//! #[derive(Serialize, Deserialize)]
//! #[xelement(name = "age")]
//! struct Age(u8);
//!
//! #[derive(Serialize, Deserialize)]
//! #[xelement(name = "person")]
//! struct Person {
//!     name: Name,
//!     age: Age,
//! }
//! ```
//!
//! The derive macros are re-exported by the `xmlity` crate in the `derive` feature, so you can use them directly from there without referring to [`xmlity_derive`].

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct _ReadMeDocTests;

use std::borrow::Cow;

use de::{DeriveDeserializationGroup, DeriveDeserialize};
use proc_macro2::Span;
use quote::{quote, ToTokens};
use ser::{DeriveSerializationGroup, DeriveSerialize, DeriveSerializeAttribute};
use syn::{parse_macro_input, DeriveInput, Expr, Ident};

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

/// Derives the [`Serialize`] trait for a type.
///
/// This macro works to serialize to XML-elements and other types of nodes including text and CDATA.
/// To serialize to attributes, use the [`SerializeAttribute`] derive macro instead.
///
/// <div style="background:rgba(120,145,255,0.45);padding:0.75em;">
/// <strong>NOTE:</strong> It is perfectly possible to derive both Serialize and SerializeAttribute for the same type, allowing the parent to decide which serialization method to use. Since deserialization can work from multiple sources, simply deriving Deserialize is sufficient to deserialize from either elements or attributes (depending on what is enabled through the derive macro).
/// </div>
///
/// ONE OF the following attributes can be applied to the root of a type to specify how the type should be serialized:
/// - `#[xelement(...)]` - Specifies that the type should be serialized as an element.
/// - `#[xvalue(...)]` - Specifies that the type should be serialized as a value.
/// - None (default) - Specifies that the type is a composite type. Currently, this is only used for enums which allow for one of the variants to be serialized as an element or value.
///
/// ## Configuration
///
/// ### #[xelement(...)]
/// The `#[xelement(...)]` attribute can be applied to the root of a type to specify that the type should be serialized as an element.
///
/// #### Options
///
/// <table style="width:100%;">
/// <thead>
/// <tr>
/// <th>Name</th>
/// <th>Type</th>
/// <th>Description</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <!--=================================================-->
/// <tr>
/// <th>
/// name
/// </th>
/// <td>
/// <code>String</code>
/// </td>
/// <td>
/// Element name.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// namespace
/// </th>
/// <td>
/// <code>String</code>
/// </td>
/// <td>
/// Must be a valid namespace string.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// namespace_expr
/// </th>
/// <td>
/// <code>Expr</code>
/// </td>
/// <td>
/// Element namespace expression. This should be a value of type `xmlity::XmlNamespace`.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// preferred_prefix
/// </th>
/// <td>
/// <code>String</code>
/// </td>
/// <td>
/// Must be a valid XML prefix.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// enforce_prefix
/// </th>
/// <td>
/// <code>bool</code>
/// </td>
/// <td>
/// Element namespace.
/// </td>
/// </tr>
/// <!--=================================================-->
/// </tbody>
/// </table>
///
/// #### Examples
///
/// <table style="width:100%;">
/// <thead>
/// <tr>
/// <th>XML</th>
/// <th>Rust types</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <tr>
/// <td>
///
/// ```xml
/// <note>
///   <to>Tove</to>
///   <from>Jani</from>
///   <heading>Reminder</heading>
///   <body>Message...</body>
/// </note>
/// ```
///
/// </td>
/// <td rowspan="3">
///
///   ```ignore
///   #[derive(Serialize)]
///   #[xelement(name = "from")]
///   struct From(String);
///
///   #[derive(Serialize)]
///   #[xelement(name = "heading")]
///   struct Heading(String);
///
///   #[derive(Serialize)]
///   #[xelement(name = "body")]
///   struct Body(String);
///
///   #[derive(Serialize)]
///   #[xelement(name = "note")]
///   struct Note {
///       to: To,
///       from: From,
///       heading: Heading,
///       body: Body,
///   }
///   ```
///
/// </td>
/// </tr>
/// <tr>
/// <th>Rust value</th>
/// </tr>
/// <tr>
/// <td>
///
///   ```ignore
///   Note {
///       to: To("Tove".to_string()),
///       from: From("Jani".to_string()),
///       heading: Heading("Reminder".to_string()),
///       body: Body("Message...".to_string()),
///   }
///   ```
///
/// </td>
/// </tr>
/// </tbody>
/// </table>
///
/// ### #[xvalue(...)]
/// The `#[xvalue(...)]` attribute can be applied to the root of a type to specify that the type should be serialized as a value. What this means differs based on the type of the root.
/// - For enums, the enum will be serialized as a value, with the variant name (or specified name) as the value.
///
/// #### Options
///
/// <table style="width:100%;">
/// <thead>
/// <tr>
/// <th>Name</th>
/// <th>Type</th>
/// <th>Description</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <!--=================================================-->
/// <tr>
/// <th>
/// rename_all
/// </th>
/// <td>
/// <code>"lowercase"</code>, <code>"UPPERCASE"</code>, <code>"PascalCase"</code>, <code>"camelCase"</code>, <code>"snake_case"</code>, <code>"SCREAMING_SNAKE_CASE"</code>, <code>"kebab-case"</code>, <code>"SCREAMING-KEBAB-CASE"</code>
/// </td>
/// <td>
/// Decides how enums should be serialized.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// serialization_format
/// </th>
/// <td>
/// <code>text</code>, <code>cdata</code>
/// </td>
/// <td>
/// Decides in what form the value should be serialized.
/// </td>
/// </tr>
/// <!--=================================================-->
/// </tbody>
/// </table>
///
/// ### No root attribute
/// If no root attribute is specified, the root will be serialized as a container with no individual serialization taking place. Instead it will defer to the fields of the root.
/// - For structs, the fields will be serialized as a sequence of elements.
/// - For enums, the active variant will be serialized as a sequence of elements.
#[proc_macro_derive(Serialize, attributes(xelement, xattribute, xgroup, xvalue))]
pub fn derive_serialize_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    DeriveSerialize::derive(item)
}

/// Derives the [`SerializeAttribute`] trait for a type.
///
/// This macro works to serialize to XML-attributes.
/// To serialize to elements, use the [`Serialize`] derive macro instead.
///
/// <div style="background:rgba(120,145,255,0.45);padding:0.75em;">
/// <strong>NOTE:</strong> It is perfectly possible to derive both Serialize and SerializeAttribute for the same type, allowing the parent to decide which serialization method to use. Since deserialization can work from multiple sources, simply deriving Deserialize is sufficient to deserialize from either elements or attributes (depending on what is enabled through the derive macro).
/// </div>
///
/// To configure the serialization, use the `#[xattribute(...)]` attribute on the root of the type. This attribute is required.
///
/// ## Configuration
///
/// ### #[xattribute(...)]
///
/// <table style="width:100%;">
/// <thead>
/// <tr><th colspan="2">
///
/// #### Options
///
/// <table style="width:100%;">
/// <thead>
/// <tr>
/// <th>Name</th>
/// <th>Type</th>
/// <th>Description</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <!--=================================================-->
/// <tr>
/// <th>
/// name
/// </th>
/// <td>
/// <code>String</code>
/// </td>
/// <td>
/// Element name.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// namespace
/// </th>
/// <td>
/// <code>String</code>
/// </td>
/// <td>
/// Must be a valid namespace string.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// namespace_expr
/// </th>
/// <td>
/// <code>Expr</code>
/// </td>
/// <td>
/// Element namespace expression. This should be a value of type `xmlity::XmlNamespace`.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// preferred_prefix
/// </th>
/// <td>
/// <code>String</code>
/// </td>
/// <td>
/// Must be a valid XML prefix.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// enforce_prefix
/// </th>
/// <td>
/// <code>bool</code>
/// </td>
/// <td>
/// Element namespace.
/// </td>
/// </tr>
/// <!--=================================================-->
/// </tbody>
/// </table>
#[proc_macro_derive(SerializeAttribute, attributes(xattribute))]
pub fn derive_serialize_attribute_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    DeriveSerializeAttribute::derive(item)
}

/// Derives the [`Deserialize`] trait for a type.
///
/// This macro supports deriving deserialization from elements, attributes and values.
///
/// One of the following can be applied to the root of a type:
/// - `#[xelement(...)]` - Specifies that the type can be deserialized as an element.
/// - `#[xvalue(...)]` - Specifies that the type can be deserialized as a value.
/// - `#[xattribute(...)]` - Specifies that the type can be deserialized as an attribute.
/// - No attribute/default behavior - Specifies that the type is a composite type. Can be deserialized from a sequence of elements.
///
/// ## Configuration
///
/// ### #[xelement(...)]
/// The `#[xelement(...)]` attribute can be applied to the root of a type to specify that the type can be deserialized from an element.
///
/// #### Options
///
/// <table style="width:100%;">
/// <thead>
/// <tr>
/// <th>Name</th>
/// <th>Type</th>
/// <th>Description</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <!--=================================================-->
/// <tr>
/// <th>
/// name
/// </th>
/// <td>
/// <code>String</code>
/// </td>
/// <td>
/// Element name.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// namespace
/// </th>
/// <td>
/// <code>String</code>
/// </td>
/// <td>
/// Must be a valid namespace string.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// namespace_expr
/// </th>
/// <td>
/// <code>Expr</code>
/// </td>
/// <td>
/// Element namespace expression. This should be a value of type `xmlity::XmlNamespace`.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// allow_unknown_children
/// </th>
/// <td>
/// <code>bool</code>
/// </td>
/// <td>
/// Element namespace.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// allow_unknown_attributes
/// </th>
/// <td>
/// <code>bool</code>
/// </td>
/// <td>
/// Element namespace.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// deserialize_any_name
/// </th>
/// <td>
/// <code>bool</code>
/// </td>
/// <td>
/// Element namespace.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// attribute_order
/// </th>
/// <td>
/// <code>"loose"</code>, <code>"none"</code>
/// </td>
/// <td>
/// Element namespace.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// children_order
/// </th>
/// <td>
/// <code>"loose"</code>, <code>"none"</code>
/// </td>
/// <td>
/// Element namespace.
/// </td>
/// </tr>
/// <!--=================================================-->
/// </tbody>
/// </table>
///
/// #### Examples
///
/// <table style="width:100%;">
/// <thead>
/// <tr>
/// <th>XML</th>
/// <th>Rust types</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <tr>
/// <td>
///
/// ```xml
/// <note>
///   <to>Tove</to>
///   <from>Jani</from>
///   <heading>Reminder</heading>
///   <body>Message...</body>
/// </note>
/// ```
///
/// </td>
/// <td rowspan="3">
///
///   ```ignore
///   #[derive(Deserialize)]
///   #[xelement(name = "from")]
///   struct From(String);
///
///   #[derive(Deserialize)]
///   #[xelement(name = "heading")]
///   struct Heading(String);
///
///   #[derive(Deserialize)]
///   #[xelement(name = "body")]
///   struct Body(String);
///
///   #[derive(Deserialize)]
///   #[xelement(name = "note")]
///   struct Note {
///       to: To,
///       from: From,
///       heading: Heading,
///       body: Body,
///   }
///   ```
///
/// </td>
/// </tr>
/// <tr>
/// <th>Rust value</th>
/// </tr>
/// <tr>
/// <td>
///
///   ```ignore
///   Note {
///       to: To("Tove".to_string()),
///       from: From("Jani".to_string()),
///       heading: Heading("Reminder".to_string()),
///       body: Body("Message...".to_string()),
///   }
///   ```
///
/// </td>
/// </tr>
/// </tbody>
/// </table>
///
/// ### #[xvalue(...)]
/// The `#[xvalue(...)]` attribute can be applied to the root of a type to specify that the type can be deserialized from a text or CDATA node.
///
/// #### Options
///
/// <table style="width:100%;">
/// <thead>
/// <tr>
/// <th>Name</th>
/// <th>Type</th>
/// <th>Description</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <!--=================================================-->
/// <tr>
/// <th>
/// rename_all
/// </th>
/// <td>
/// <code>"lowercase"</code>, <code>"UPPERCASE"</code>, <code>"PascalCase"</code>, <code>"camelCase"</code>, <code>"snake_case"</code>, <code>"SCREAMING_SNAKE_CASE"</code>, <code>"kebab-case"</code>, <code>"SCREAMING-KEBAB-CASE"</code>
/// </td>
/// <td>
/// Decides how enums should be deserialized.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// serialization_format
/// </th>
/// <td>
/// <code>text</code>, <code>cdata</code>
/// </td>
/// <td>
/// Decides in what form the value should be deserialized from.
/// </td>
/// </tr>
/// <!--=================================================-->
/// </tbody>
/// </table>
///
/// ### #[xattribute(...)]
/// The `#[xattribute(...)]` attribute can be applied to the root of a type to specify that the type can be deserialized from an attribute.
///
/// #### Options
///
/// <table style="width:100%;">
/// <thead>
/// <tr>
/// <th>Name</th>
/// <th>Type</th>
/// <th>Description</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <!--=================================================-->
/// <tr>
/// <th>
/// name
/// </th>
/// <td>
/// <code>String</code>
/// </td>
/// <td>
/// Element name.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// namespace
/// </th>
/// <td>
/// <code>String</code>
/// </td>
/// <td>
/// Must be a valid namespace string.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// namespace_expr
/// </th>
/// <td>
/// <code>Expr</code>
/// </td>
/// <td>
/// Element namespace expression. This should be a value of type `xmlity::XmlNamespace`.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// deserialize_any_name
/// </th>
/// <td>
/// <code>bool</code>
/// </td>
/// <td>
/// Element namespace.
/// </td>
/// </tr>
/// <!--=================================================-->
/// </tbody>
/// </table>
///
/// ### No attribute
/// If no attribute is specified, the type will be deserialized from a sequence. Of note is that enums will try to deserialize each variant in order, and the first one that succeeds will be used. This allows for a form of trial-and-error deserialization which can be useful in many situations, including supporting multiple types of elements or falling back to an [`xmlity::XmlValue`] in case of an unknown element.
#[proc_macro_derive(Deserialize, attributes(xelement, xattribute, xgroup, xvalue))]
pub fn derive_deserialize_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    DeriveDeserialize::derive(item)
}

/// Derives the [`SerializationGroup`] trait for a type.
///
/// To configure the serialization, use the `#[xgroup(...)]` attribute on the root of the type.
///
/// <div style="background:rgba(120,145,255,0.45);padding:0.75em;">
/// <strong>NOTE:</strong> This trait/attribute is not mutually exclusive with the Serialize trait/attributes. This means that you could for example use a struct both as a sequence (Serialize with no attribute) and as a group (SerializationGroup with the attribute).
/// </div>
///
/// ## Configuration
///
/// ### #[xgroup(...)]
///
/// #### Options
///
/// None for serialization currently.
#[proc_macro_derive(SerializationGroup, attributes(xvalue, xattribute, xgroup))]
pub fn derive_serialization_group_attribute_fn(
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    DeriveSerializationGroup::derive(item)
}

/// Derives the [`DeserializationGroup`] trait for a type.
///
/// To configure the deserialization, use the `#[xgroup(...)]` attribute on the root of the type.
///
/// <div style="background:rgba(120,145,255,0.45);padding:0.75em;">
/// <strong>NOTE:</strong> This trait/attribute is not mutually exclusive with the [Deserialize] trait/attribute. This means that you could for example use a struct both as a sequence ([Deserialize] with no attribute) and as a group ([DeserializationGroup] with the attribute).
/// </div>
///
/// [Deserialize]: Deserialize
/// [DeserializationGroup]: DeserializationGroup
///
/// ## Configuration
///
/// ### #[xgroup(...)]
///
/// #### Options
///
/// <table style="width:100%;">
/// <thead>
/// <tr>
/// <th>Name</th>
/// <th>Type</th>
/// <th>Description</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <!--=================================================-->
/// <tr>
/// <th>
/// attribute_order
/// </th>
/// <td>
/// <code>"strict"</code>, <code>"loose"</code>, <code>"none"</code>
/// </td>
/// <td>
/// Element namespace.
/// </td>
/// </tr>
/// <!--=================================================-->
/// <tr>
/// <th>
/// children_order
/// </th>
/// <td>
/// <code>"strict"</code>, <code>"loose"</code>, <code>"none"</code>
/// </td>
/// <td>
/// Element namespace.
/// </td>
/// </tr>
/// <!--=================================================-->
/// </tbody>
/// </table>
#[proc_macro_derive(DeserializationGroup, attributes(xvalue, xattribute, xgroup))]
pub fn derive_deserialization_group_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    DeriveDeserializationGroup::derive(item)
}

#[derive(Clone)]
enum FieldIdent {
    Named(syn::Ident),
    Indexed(syn::Index),
}

impl FieldIdent {
    pub fn to_named_ident(&self) -> Cow<'_, syn::Ident> {
        match self {
            FieldIdent::Named(ident) => Cow::Borrowed(ident),
            FieldIdent::Indexed(index) => Cow::Owned(Ident::new(
                format!("__{}", index.index).as_str(),
                Span::call_site(),
            )),
        }
    }
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

impl XmlNamespaceRef<'_> {
    fn into_owned(self) -> XmlNamespaceRef<'static> {
        match self {
            XmlNamespaceRef::Static(namespace) => XmlNamespaceRef::Static(namespace.into_owned()),
            XmlNamespaceRef::Dynamic(expr) => XmlNamespaceRef::Dynamic(expr.to_owned()),
        }
    }
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

    fn into_owned(self) -> ExpandedName<'static> {
        ExpandedName {
            name: self.name.into_owned(),
            namespace: self.namespace.map(|namespace| namespace.into_owned()),
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
