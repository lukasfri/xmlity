use std::borrow::Cow;

use darling::FromMeta;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_quote, Expr, Ident, Index, ItemStruct, Stmt, Type, Visibility};

use crate::{
    derive::{DeriveError, DeriveResult},
    options::{records::fields::FieldOpts, FieldWithOpts},
};

#[derive(Clone)]
pub enum FieldIdent {
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

#[derive(Debug, Clone)]
pub enum XmlNamespaceRef<'a> {
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

#[derive(Debug, Clone)]
pub struct LocalName<'a>(pub Cow<'a, str>);

impl LocalName<'_> {
    pub fn into_owned(self) -> LocalName<'static> {
        LocalName(Cow::Owned(self.0.to_string()))
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

impl XmlNamespace<'_> {
    pub fn into_owned(self) -> XmlNamespace<'static> {
        XmlNamespace(Cow::Owned(self.0.to_string()))
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

#[derive(Debug, Clone)]
pub struct ExpandedName<'a> {
    name: LocalName<'a>,
    namespace: Option<XmlNamespaceRef<'a>>,
}

impl<'a> ExpandedName<'a> {
    pub fn new(name: LocalName<'a>, namespace: Option<XmlNamespace<'a>>) -> Self {
        Self {
            name,
            namespace: namespace.map(XmlNamespaceRef::Static),
        }
    }

    pub fn new_ref(name: LocalName<'a>, namespace: Option<Expr>) -> Self {
        Self {
            name,
            namespace: namespace.map(XmlNamespaceRef::Dynamic),
        }
    }

    pub fn into_owned(self) -> ExpandedName<'static> {
        ExpandedName {
            name: self.name.into_owned(),
            namespace: self.namespace.map(|namespace| namespace.into_owned()),
        }
    }

    pub fn to_expression(Self { name, namespace }: &Self) -> Expr {
        let xml_namespace: Expr = match namespace {
            Some(xml_namespace) => {
                parse_quote! { ::core::option::Option::Some(#xml_namespace) }
            }
            None => parse_quote! { ::core::option::Option::None },
        };

        parse_quote! {
            ::xmlity::ExpandedName::new(#name, #xml_namespace)
        }
    }
}

impl quote::ToTokens for ExpandedName<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(Self::to_expression(self).to_token_stream())
    }
}

pub fn non_bound_generics(generics: &syn::Generics) -> syn::Generics {
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

#[derive(Clone, Copy, PartialEq)]
pub enum StructType {
    Named,
    Unnamed,
    Unit,
}

#[derive(Clone)]
pub enum StructTypeWithFields<N, U> {
    Named(N),
    Unnamed(U),
    Unit,
}

pub struct RecordInput<'a, T: Fn(syn::Expr) -> syn::Expr> {
    pub impl_for_ident: Cow<'a, syn::Ident>,
    pub constructor_path: Cow<'a, syn::Path>,
    pub result_type: Cow<'a, syn::Type>,
    pub generics: Cow<'a, syn::Generics>,
    pub wrapper_function: T,
    pub record_path: Cow<'a, syn::Expr>,
    pub sub_path_ident: Option<Ident>,
    #[allow(clippy::type_complexity)]
    pub fields: StructTypeWithFields<
        Vec<FieldWithOpts<syn::Ident, FieldOpts>>,
        Vec<FieldWithOpts<syn::Index, FieldOpts>>,
    >,
    // True if the record is an enum variant with more than one field
    pub fallable_deconstruction: bool,
}

#[allow(clippy::type_complexity)]
pub fn fields_with_opts(
    fields: &syn::Fields,
) -> DeriveResult<
    StructTypeWithFields<
        Vec<FieldWithOpts<syn::Ident, FieldOpts>>,
        Vec<FieldWithOpts<syn::Index, FieldOpts>>,
    >,
> {
    match fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .map(|f| {
                let field_ident = f.ident.clone().expect("Named struct");

                DeriveResult::Ok(FieldWithOpts {
                    field_ident,
                    options: FieldOpts::from_field(f)?,
                    field_type: f.ty.clone(),
                })
            })
            .collect::<Result<Vec<_>, DeriveError>>()
            .map(StructTypeWithFields::Named),
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| {
                DeriveResult::Ok(FieldWithOpts {
                    field_ident: syn::Index::from(i),
                    options: FieldOpts::from_field(f)?,
                    field_type: f.ty.clone(),
                })
            })
            .collect::<Result<Vec<_>, DeriveError>>()
            .map(StructTypeWithFields::Unnamed),
        _ => Ok(StructTypeWithFields::Unit),
    }
}

pub fn parse_struct_derive_input(
    input: &syn::DeriveInput,
) -> Result<RecordInput<'_, impl Fn(syn::Expr) -> syn::Expr + '_>, DeriveError> {
    let ident = &input.ident;
    let generics = &input.generics;
    Ok(RecordInput {
        impl_for_ident: Cow::Borrowed(ident),
        constructor_path: Cow::Owned(parse_quote!(#ident)),
        result_type: Cow::Owned(parse_quote! { #ident }),
        generics: Cow::Borrowed(generics),
        record_path: Cow::Owned(parse_quote!(self)),
        wrapper_function: std::convert::identity,
        fields: match &input.data {
            syn::Data::Struct(data_struct) => fields_with_opts(&data_struct.fields)?,
            _ => panic!("Wrong options. Only structs can be used for xelement."),
        },
        fallable_deconstruction: false,
        sub_path_ident: None,
    })
}

pub fn parse_enum_variant_derive_input<'a>(
    enum_ident: &'a syn::Ident,
    enum_generics: &'a syn::Generics,
    variant: &'a syn::Variant,
    fallible_enum: bool,
) -> Result<RecordInput<'a, impl Fn(syn::Expr) -> syn::Expr + 'a>, DeriveError> {
    let variant_ident = &variant.ident;
    let variant_wrapper_ident = Ident::new(
        &format!("__XmlityVariant__{variant_ident}"),
        variant_ident.span(),
    );
    let sub_value_ident = Ident::new("__inner", Span::call_site());
    let variant_wrapper_ident2 = variant_wrapper_ident.clone();
    let sub_value_ident2 = sub_value_ident.clone();
    let wrapper_function = move |data| {
        parse_quote!(
            #variant_wrapper_ident {
                #sub_value_ident: #data,
            }
        )
    };

    let result_generics = non_bound_generics(enum_generics);

    Ok(RecordInput {
        impl_for_ident: Cow::Owned(variant_wrapper_ident2),
        constructor_path: Cow::Owned(parse_quote!(#enum_ident::#variant_ident)),
        result_type: Cow::Owned(parse_quote! { #enum_ident #result_generics }),
        generics: Cow::Borrowed(enum_generics),
        wrapper_function,
        record_path: Cow::Owned(parse_quote!(self.#sub_value_ident2)),
        fields: fields_with_opts(&variant.fields)?,
        fallable_deconstruction: fallible_enum,
        sub_path_ident: Some(sub_value_ident2),
    })
}

fn named_constructor_expr<K: ToTokens, V: ToTokens>(
    ident: &syn::Path,
    fields: impl IntoIterator<Item = (K, V)>,
) -> Expr {
    let field_tokens = fields.into_iter().map(|(ident, expression)| {
        quote! {
            #ident: #expression,
        }
    });

    parse_quote! {
        #ident {
            #(#field_tokens)*
        }
    }
}

fn unnamed_constructor_expr<T: ToTokens>(
    ident: &syn::Path,
    fields: impl IntoIterator<Item = T>,
) -> Expr {
    let fields = fields.into_iter();

    parse_quote! {
      #ident (
        #(#fields,)*
    )
    }
}

pub fn constructor_expr<T: ToTokens>(
    ident: &syn::Path,
    fields: impl IntoIterator<Item = (FieldIdent, T)>,
    constructor_type: &StructType,
) -> Expr {
    let mut fields = fields.into_iter();
    match constructor_type {
        StructType::Unnamed => {
            unnamed_constructor_expr(ident, fields.map(|(_, value_expression)| value_expression))
        }
        StructType::Named => named_constructor_expr(
            ident,
            fields.filter_map(|(a, value_expression)| match a {
                FieldIdent::Named(field_ident) => Some((field_ident, value_expression)),
                FieldIdent::Indexed(_) => None,
            }),
        ),
        StructType::Unit => {
            assert!(fields.next().is_none(), "unit structs cannot have fields");
            parse_quote! { #ident }
        }
    }
}

fn named_struct_definition_expr<I: ToTokens, K: ToTokens, V: ToTokens>(
    ident: I,
    generics: Option<&syn::Generics>,
    fields: impl IntoIterator<Item = (K, V)>,
    visibility: &Visibility,
) -> ItemStruct {
    let field_tokens = fields
        .into_iter()
        .map::<syn::Field, _>(|(ident, expression)| {
            parse_quote! {
                #ident: #expression
            }
        });

    parse_quote! {
        #visibility struct #ident #generics {
            #(#field_tokens,)*
        }
    }
}

fn unnamed_struct_definition_expr<I: ToTokens, T: ToTokens>(
    ident: I,
    generics: Option<&syn::Generics>,
    fields: impl IntoIterator<Item = T>,
    visibility: &Visibility,
) -> ItemStruct {
    let fields = fields.into_iter();

    parse_quote! {
        #visibility struct #ident #generics (
            #(#fields),*
        );
    }
}

fn unit_struct_definition_expr<I: ToTokens>(
    ident: I,
    generics: Option<&syn::Generics>,
    visibility: &Visibility,
) -> ItemStruct {
    parse_quote! {
        #visibility struct #ident #generics;
    }
}

pub fn struct_definition_expr<I: ToTokens>(
    ident: I,
    generics: Option<&syn::Generics>,
    fields: impl IntoIterator<Item = (FieldIdent, Type)>,
    constructor_type: &StructType,
    visibility: &Visibility,
) -> ItemStruct {
    let mut fields = fields.into_iter();
    match constructor_type {
        StructType::Unnamed => unnamed_struct_definition_expr(
            ident,
            generics,
            fields.map(|(_, value_expression)| value_expression),
            visibility,
        ),
        StructType::Named => named_struct_definition_expr(
            ident,
            generics,
            fields.filter_map(|(a, value_expression)| match a {
                FieldIdent::Named(field_ident) => Some((field_ident, value_expression)),
                FieldIdent::Indexed(_) => None,
            }),
            visibility,
        ),
        StructType::Unit => {
            assert!(fields.next().is_none(), "unit structs cannot have fields");
            unit_struct_definition_expr(ident, generics, visibility)
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn value_deconstructor(
    path: &syn::Path,
    value_expr: &syn::Expr,
    fields: &StructTypeWithFields<
        Vec<FieldWithOpts<Ident, FieldOpts>>,
        Vec<FieldWithOpts<Index, FieldOpts>>,
    >,
    fallible: bool,
) -> Vec<Stmt> {
    let fallible = if fallible {
        quote!(
            else {
                unreachable!("Internal expectation failed. This is a bug in xmlity. Please report it.")
            }
        )
    } else {
        TokenStream::new()
    };

    let fields = match fields {
        StructTypeWithFields::Named(fields) => {
            let field_deconstructor = fields.iter().map(|f| &f.field_ident);

            quote! {{ #(#field_deconstructor),* }}
        }
        StructTypeWithFields::Unnamed(fields) => {
            let field_deconstructor = fields.iter().map(|f| {
                FieldIdent::Indexed(f.field_ident.clone())
                    .to_named_ident()
                    .into_owned()
            });
            quote! { ( #(#field_deconstructor),* ) }
        }
        StructTypeWithFields::Unit => quote!(),
    };

    parse_quote! {
        let #path #fields = #value_expr #fallible;
    }
}
