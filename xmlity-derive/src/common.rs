use std::borrow::Cow;

use darling::FromMeta;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{parse_quote, Expr, Ident};

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

#[derive(Clone, Copy)]
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
