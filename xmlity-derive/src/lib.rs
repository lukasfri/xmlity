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

use de::{DeriveDeserializationGroup, DeriveDeserialize};
use quote::{quote, ToTokens};
use ser::{DeriveSerializationGroup, DeriveSerialize, DeriveSerializeAttribute};
use syn::{parse_macro_input, DeriveInput, Expr};

mod de;
mod options;
use options::{
    LocalName, XmlNamespace, XmlityFieldAttributeDeriveOpts, XmlityFieldGroupDeriveOpts,
    XmlityFieldValueDeriveOpts, XmlityRootAttributeDeriveOpts, XmlityRootElementDeriveOpts,
    XmlityRootValueDeriveOpts,
};
mod ser;
mod utils;

enum DeriveError {
    Darling(darling::Error),
    Custom(String),
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
            DeriveError::Custom(e) => {
                syn::Error::new(proc_macro2::Span::call_site(), e).to_compile_error()
            }
        }
    }

    fn custom<T: Into<String>>(error: T) -> Self {
        Self::Custom(error.into())
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
/// This macro works to serialize to XML-elements and other types of nodes including TEXT and CDATA.
/// To serialize to attributes, use the [`SerializeAttribute`] derive macro instead.
///
/// <div style="background:rgba(120,145,255,0.45);padding:0.75em;">
/// <strong>NOTE:</strong> It is perfectly possible to derive both [`Serialize`] and [`SerializeAttribute`] for the same type, allowing the parent to decide which serialization method to use. Since deserialization can work from multiple sources, simply deriving [`Deserialize`] is sufficient to deserialize from either elements or attributes (depending on what is enabled through the derive macro).
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
/// <!-- Styling of docs inspired by quick-xml's docs :) -->
/// <table style="width:100%;">
/// <thead>
/// <tr><th colspan="2">
///
/// #### Basics
///
/// </th></tr>
/// <tr>
/// <th>To parse all these XML's...</th>
/// <th>...use these Rust type(s)</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <tr>
/// <td>
/// A2
/// </td>
/// <td>
/// A3
/// <div style="background:rgba(120,145,255,0.45);padding:0.75em;">
/// A4
/// </div>
/// </td>
/// </tr>
/// </tbody>
/// </table>
///
/// ### #[xvalue(...)]
/// The `#[xvalue(...)]` attribute can be applied to the root of a type to specify that the type should be serialized as a value. What this means differs based on the type of the root.
/// - For enums, the enum will be serialized as a value, with the variant name (or specified name) as the value.
///
/// <!-- Styling of docs inspired by quick-xml's docs :) -->
/// <table style="width:100%;">
/// <thead>
/// <tr><th colspan="2">
///
/// #### Basics
///
/// </th></tr>
/// <tr>
/// <th>To parse all these XML's...</th>
/// <th>...use these Rust type(s)</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <tr>
/// <td>
/// A2
/// </td>
/// <td>
/// A3
/// <div style="background:rgba(120,145,255,0.45);padding:0.75em;">
/// A4
/// </div>
/// </td>
/// </tr>
/// </tbody>
/// </table>
///
/// ### No root attribute
/// If no root attribute is specified, the root will be serialized as a container with no individual serialization taking place. Instead it will defer to the fields of the root.
/// - For structs, the fields will be serialized as a sequence of elements.
/// - For enums, the active variant will be serialized as a sequence of elements.
///
/// <!-- Styling of docs inspired by quick-xml's docs :) -->
/// <table style="width:100%;">
/// <thead>
/// <tr><th colspan="2">
///
/// #### Basics
///
/// </th></tr>
/// <tr>
/// <th>To parse all these XML's...</th>
/// <th>...use these Rust type(s)</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <tr>
/// <td>
/// A2
/// </td>
/// <td>
/// A3
/// <div style="background:rgba(120,145,255,0.45);padding:0.75em;">
/// A4
/// </div>
/// </td>
/// </tr>
/// </tbody>
/// </table>
#[proc_macro_derive(Serialize, attributes(xelement, xattribute, xgroup, xvalue))]
pub fn derive_serialize_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    DeriveSerialize::derive(item)
}

/// Derives the [`SerializeAttribute`] trait for a type.
///
/// This macro works to serialize to XML-attributes.
/// To serialize to elements, use the [`Serialize`] derive macro instead.
///
/// To configure the serialization, use the `#[xattribute(...)]` attribute on the root of the type. This attribute is required.
///
/// ## Configuration
///
/// ### #[xattribute(...)]
///
/// <!-- Styling of docs inspired by quick-xml's docs :) -->
/// <table style="width:100%;">
/// <thead>
/// <tr><th colspan="2">
///
/// #### Basics
///
/// </th></tr>
/// <tr>
/// <th>To parse all these XML's...</th>
/// <th>...use these Rust type(s)</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <tr>
/// <td>
/// A2
/// </td>
/// <td>
/// A3
/// <div style="background:rgba(120,145,255,0.45);padding:0.75em;">
/// A4
/// </div>
/// </td>
/// </tr>
/// </tbody>
/// </table>
#[proc_macro_derive(SerializeAttribute, attributes(xattribute))]
pub fn derive_serialize_attribute_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    DeriveSerializeAttribute::derive(item)
}

enum DeriveDeserializeOption {
    None,
    Element(XmlityRootElementDeriveOpts),
    Attribute(XmlityRootAttributeDeriveOpts),
    Value(XmlityRootValueDeriveOpts),
}

impl DeriveDeserializeOption {
    pub fn parse(ast: &DeriveInput) -> Result<Self, DeriveError> {
        let element_opts = XmlityRootElementDeriveOpts::parse(ast)?;
        let attribute_opts = XmlityRootAttributeDeriveOpts::parse(ast)?;
        let value_opts = XmlityRootValueDeriveOpts::parse(ast)?;

        let opts = match (element_opts, attribute_opts, value_opts) {
            (Some(element_opts), None, None) => DeriveDeserializeOption::Element(element_opts),
            (None, Some(attribute_opts), None) => DeriveDeserializeOption::Attribute(attribute_opts),
            (None, None, Some(value_opts)) => DeriveDeserializeOption::Value(value_opts),
            (None, None, None) => DeriveDeserializeOption::None,
            _ => panic!("Wrong options. Only one of xelement, xattribute, or xvalue can be used for root elements."),
        };
        Ok(opts)
    }
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
/// <!-- Styling of docs inspired by quick-xml's docs :) -->
/// <table style="width:100%;">
/// <thead>
/// <tr><th colspan="2">
///
/// #### Basics
///
/// </th></tr>
/// <tr>
/// <th>To parse all these XML's...</th>
/// <th>...use these Rust type(s)</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <tr>
/// <td>
/// A2
/// </td>
/// <td>
/// A3
/// <div style="background:rgba(120,145,255,0.45);padding:0.75em;">
/// A4
/// </div>
/// </td>
/// </tr>
/// </tbody>
/// </table>
///
/// ### #[xvalue(...)]
/// The `#[xvalue(...)]` attribute can be applied to the root of a type to specify that the type can be deserialized from a TEXT or CDATA node.
///
/// <!-- Styling of docs inspired by quick-xml's docs :) -->
/// <table style="width:100%;">
/// <thead>
/// <tr><th colspan="2">
///
/// #### Basics
///
/// </th></tr>
/// <tr>
/// <th>To parse all these XML's...</th>
/// <th>...use these Rust type(s)</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <tr>
/// <td>
/// A2
/// </td>
/// <td>
/// A3
/// <div style="background:rgba(120,145,255,0.45);padding:0.75em;">
/// A4
/// </div>
/// </td>
/// </tr>
/// </tbody>
/// </table>
///
/// ### #[xattribute(...)]
/// The `#[xattribute(...)]` attribute can be applied to the root of a type to specify that the type can be deserialized from an attribute.
///
/// <!-- Styling of docs inspired by quick-xml's docs :) -->
/// <table style="width:100%;">
/// <thead>
/// <tr><th colspan="2">
///
/// #### Basics
///
/// </th></tr>
/// <tr>
/// <th>To parse all these XML's...</th>
/// <th>...use these Rust type(s)</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <tr>
/// <td>
/// A2
/// </td>
/// <td>
/// A3
/// <div style="background:rgba(120,145,255,0.45);padding:0.75em;">
/// A4
/// </div>
/// </td>
/// </tr>
/// </tbody>
/// </table>
///
/// ### No attribute
/// If no attribute is specified, the type will be deserialized from a sequence. Of note is that enums will try to deserialize each variant in order, and the first one that succeeds will be used. This allows for a form of trial-and-error deserialization which can be useful in many situations, including supporting multiple types of elements or falling back to an [`xmlity::XmlValue`] in case of an unknown element.
///
/// <!-- Styling of docs inspired by quick-xml's docs :) -->
/// <table style="width:100%;">
/// <thead>
/// <tr><th colspan="2">
///
/// #### Basics
///
/// </th></tr>
/// <tr>
/// <th>To parse all these XML's...</th>
/// <th>...use these Rust type(s)</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <tr>
/// <td>
/// A2
/// </td>
/// <td>
/// A3
/// <div style="background:rgba(120,145,255,0.45);padding:0.75em;">
/// A4
/// </div>
/// </td>
/// </tr>
/// </tbody>
/// </table>
#[proc_macro_derive(Deserialize, attributes(xelement, xattribute, xgroup, xvalue))]
pub fn derive_deserialize_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    DeriveDeserialize::derive(item)
}

/// Derives the [`SerializationGroup`] trait for a type.
///
/// To configure the serialization, use the `#[xgroup(...)]` attribute on the root of the type. This trait/attribute is not mutually exclusive with the [`Serialize`] trait/attributes. This means that you could for example use a struct both as a sequence (Serialize with no attribute) and as a group (SerializationGroup with the attribute).
///
/// ## Configuration
///
/// ### #[xgroup(...)]
///
/// <!-- Styling of docs inspired by quick-xml's docs :) -->
/// <table style="width:100%;">
/// <thead>
/// <tr><th colspan="2">
///
/// #### Basics
///
/// </th></tr>
/// <tr>
/// <th>To parse all these XML's...</th>
/// <th>...use these Rust type(s)</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <tr>
/// <td>
/// A2
/// </td>
/// <td>
/// A3
/// <div style="background:rgba(120,145,255,0.45);padding:0.75em;">
/// A4
/// </div>
/// </td>
/// </tr>
/// </tbody>
/// </table>
#[proc_macro_derive(SerializationGroup, attributes(xelement, xattribute, xgroup))]
pub fn derive_serialization_group_attribute_fn(
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    DeriveSerializationGroup::derive(item)
}

/// Derives the [`DeserializationGroup`] trait for a type.
///
/// To configure the deserialization, use the `#[xgroup(...)]` attribute on the root of the type. This trait/attribute is not mutually exclusive with the [`Deserialize`] trait/attribute. This means that you could for example use a struct both as a sequence (Deserialize with no attribute) and as a group (DeserializationGroup with the attribute).
///
/// ## Configuration
///
/// ### #[xgroup(...)]
///
/// <!-- Styling of docs inspired by quick-xml's docs :) -->
/// <table style="width:100%;">
/// <thead>
/// <tr><th colspan="2">
///
/// #### Basics
///
/// </th></tr>
/// <tr>
/// <th>To parse all these XML's...</th>
/// <th>...use these Rust type(s)</th>
/// </tr>
/// </thead>
/// <tbody style="vertical-align:top;">
/// <tr>
/// <td>
/// A2
/// </td>
/// <td>
/// A3
/// <div style="background:rgba(120,145,255,0.45);padding:0.75em;">
/// A4
/// </div>
/// </td>
/// </tr>
/// </tbody>
/// </table>
#[proc_macro_derive(DeserializationGroup, attributes(xelement, xattribute, xgroup))]
pub fn derive_deserialization_group_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    DeriveDeserializationGroup::derive(item)
}

fn simple_compile_error(text: &str) -> proc_macro2::TokenStream {
    quote! {
        compile_error!(#text);
    }
}

#[derive(Clone)]
enum XmlityFieldDeriveOpts {
    Value(XmlityFieldValueDeriveOpts),
    Attribute(XmlityFieldAttributeDeriveOpts),
    Group(XmlityFieldGroupDeriveOpts),
}

#[derive(Clone)]
enum XmlityFieldAttributeGroupDeriveOpts {
    Attribute(XmlityFieldAttributeDeriveOpts),
    Group(XmlityFieldGroupDeriveOpts),
}

#[derive(Clone)]
enum XmlityFieldValueGroupDeriveOpts {
    Value(XmlityFieldValueDeriveOpts),
    Group(XmlityFieldGroupDeriveOpts),
}

impl XmlityFieldDeriveOpts {
    fn from_field(field: &syn::Field) -> Result<Self, DeriveError> {
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

#[derive(Clone)]
struct DeserializeBuilderField<BuilderFieldIdent, OptionType> {
    builder_field_ident: BuilderFieldIdent,
    // If the field is indexed, this is none.
    field_ident: FieldIdent,
    field_type: syn::Type,
    options: OptionType,
}

impl<A, T> DeserializeBuilderField<A, T> {
    pub fn map_options<U, F: FnOnce(T) -> U>(self, f: F) -> DeserializeBuilderField<A, U> {
        DeserializeBuilderField {
            builder_field_ident: self.builder_field_ident,
            field_ident: self.field_ident,
            field_type: self.field_type,
            options: f(self.options),
        }
    }

    pub fn map_options_opt<U, F: FnOnce(T) -> Option<U>>(
        self,
        f: F,
    ) -> Option<DeserializeBuilderField<A, U>> {
        f(self.options).map(|options| DeserializeBuilderField {
            builder_field_ident: self.builder_field_ident,
            field_ident: self.field_ident,
            field_type: self.field_type,
            options,
        })
    }
}

#[derive(Clone)]
struct SerializeField<OptionType> {
    // If the field is indexed, this is none.
    field_ident: FieldIdent,
    field_type: syn::Type,
    options: OptionType,
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
