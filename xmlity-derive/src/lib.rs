//! # XMLity Derive
//!
//! This crate contains the proc-macros for XMLity, specifically the derive macros for [`Serialize`], [`SerializeAttribute`], [`Deserialize`], [`SerializationGroup`], and [`DeserializationGroup`].

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct _ReadMeDocTests;

use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod de;
mod options;
use options::{
    XmlityFieldAttributeDeriveOpts, XmlityFieldElementDeriveOpts, XmlityFieldGroupDeriveOpts,
    XmlityRootAttributeDeriveOpts, XmlityRootElementDeriveOpts, XmlityRootGroupDeriveOpts,
    XmlityRootValueDeriveOpts,
};
mod ser;
mod utils;

/// Derives the [`xmlity::Serialize`] trait for a type.
#[proc_macro_derive(Serialize, attributes(xelement, xattribute, xgroup, xvalue))]
pub fn derive_serialize_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let element_opts = XmlityRootElementDeriveOpts::parse(&ast).expect("Wrong options");
    let value_opts = XmlityRootValueDeriveOpts::parse(&ast).expect("Wrong options");

    ser::derive_element_serialize_fn(ast, element_opts.as_ref(), value_opts.as_ref())
        .expect("Wrong options")
        .into()
}

/// Derives the [`xmlity::SerializeAttribute`] trait for a type.
#[proc_macro_derive(SerializeAttribute, attributes(xattribute))]
pub fn derive_serialize_attribute_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let opts = XmlityRootAttributeDeriveOpts::parse(&ast)
        .expect("Wrong options")
        .unwrap_or_default();

    ser::derive_attribute_serialize_fn(ast, opts).into()
}

/// Derives the [`xmlity::Deserialize`] trait for a type.
#[proc_macro_derive(Deserialize, attributes(xelement, xattribute, xgroup, xvalue))]
pub fn derive_deserialize_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let element_opts = XmlityRootElementDeriveOpts::parse(&ast).expect("Wrong options");
    let attribute_opts = XmlityRootAttributeDeriveOpts::parse(&ast).expect("Wrong options");
    let value_opts = XmlityRootValueDeriveOpts::parse(&ast).expect("Wrong options");

    de::derive_deserialize_fn(ast, element_opts, attribute_opts, value_opts)
        .expect("Wrong options")
        .into()
}

/// Derives the [`xmlity::SerializationGroup`] trait for a type.
#[proc_macro_derive(SerializationGroup, attributes(xelement, xattribute, xgroup))]
pub fn derive_serialization_group_attribute_fn(
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let opts = XmlityRootGroupDeriveOpts::parse(&ast)
        .expect("Wrong options")
        .unwrap_or_default();

    ser::derive_group_serialize_fn(ast, opts)
        .expect("Wrong options")
        .into()
}

/// Derives the [`xmlity::DeserializationGroup`] trait for a type.
#[proc_macro_derive(DeserializationGroup, attributes(xelement, xattribute, xgroup))]
pub fn derive_deserialization_group_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let opts = XmlityRootGroupDeriveOpts::parse(&ast)
        .expect("Wrong options")
        .unwrap_or_default();

    de::derive_deserialization_group_fn(ast, opts)
        .expect("Wrong options")
        .into()
}

fn simple_compile_error(text: &str) -> proc_macro2::TokenStream {
    quote! {
        compile_error!(#text);
    }
}

#[derive(Clone)]
enum XmlityFieldDeriveOpts {
    Element(XmlityFieldElementDeriveOpts),
    Attribute(XmlityFieldAttributeDeriveOpts),
    Group(XmlityFieldGroupDeriveOpts),
}

#[derive(Clone)]
enum XmlityFieldAttributeGroupDeriveOpts {
    Attribute(XmlityFieldAttributeDeriveOpts),
    Group(XmlityFieldGroupDeriveOpts),
}

#[derive(Clone)]
enum XmlityFieldElementGroupDeriveOpts {
    Element(XmlityFieldElementDeriveOpts),
    Group(XmlityFieldGroupDeriveOpts),
}

impl XmlityFieldDeriveOpts {
    fn from_field(field: &syn::Field) -> Result<Self, darling::Error> {
        let element = XmlityFieldElementDeriveOpts::from_field(field)?;
        let attribute = XmlityFieldAttributeDeriveOpts::from_field(field)?;
        let group = XmlityFieldGroupDeriveOpts::from_field(field)?;
        Ok(match (element, attribute, group) {
            (Some(element), None, None) => Self::Element(element),
            (None, Some(attribute), None) => Self::Attribute(attribute),
            (None, None, Some(group)) => Self::Group(group),
            (None, None, None) => Self::Element(XmlityFieldElementDeriveOpts::default()),
            _ => {
                return Err(darling::Error::custom(
                    "Cannot have multiple xmlity field attributes on the same field",
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

struct ExpandedName<'a> {
    name: &'a str,
    namespace: Option<&'a str>,
}

impl<'a> ExpandedName<'a> {
    fn new(name: &'a str, namespace: Option<&'a str>) -> Self {
        Self { name, namespace }
    }

    fn to_expression(Self { name, namespace }: &Self) -> proc_macro2::TokenStream {
        let xml_namespace = match namespace {
            Some(xml_namespace) => {
                quote! { ::core::option::Option::Some(<::xmlity::XmlNamespace as ::core::str::FromStr>::from_str(#xml_namespace).expect("XML namespace in derive macro is invalid. This is a bug in xmlity. Please report it.")) }
            }
            None => quote! { ::core::option::Option::None },
        };

        quote! {
            ::xmlity::ExpandedName::new(<::xmlity::LocalName as ::core::str::FromStr>::from_str(#name).expect("XML name in derive macro is invalid. This is a bug in xmlity. Please report it."), #xml_namespace)
        }
    }
}

impl quote::ToTokens for ExpandedName<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(Self::to_expression(self))
    }
}
