#![allow(clippy::type_complexity)]

use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use syn::{parse_quote, Expr, ExprWhile, Ident, Lifetime, LifetimeParam, Stmt, Type};

use crate::{
    common::{non_bound_generics, ExpandedName},
    de::builders::{DeserializeBuilder, VisitorBuilder, VisitorBuilderExt},
    options::{records, Extendable},
    DeriveError,
};

use super::{RecordDeserializeBuilder, RecordInput};

pub struct DeserializeVariantBuilder<'a, T: Fn(syn::Expr) -> syn::Expr> {
    pub record: &'a RecordInput<'a, T>,
    pub opts: &'a records::roots::DeserializeRootOpts,
}

impl<'a, T: Fn(syn::Expr) -> syn::Expr> DeserializeVariantBuilder<'a, T> {
    pub fn new(record: &'a RecordInput<T>, opts: &'a records::roots::DeserializeRootOpts) -> Self {
        Self { record, opts }
    }

    fn value_access_ident(&self) -> Ident {
        Ident::new("__value", Span::call_site())
    }

    pub fn definition(&self) -> syn::ItemStruct {
        let Self { record, .. } = self;

        let ident = record.impl_for_ident.as_ref();
        let enum_type = record.result_type.as_ref();

        let value_access_ident = self.value_access_ident();

        parse_quote! {
            struct #ident {
                #value_access_ident: #enum_type,
            }
        }
    }
}

impl<T: Fn(syn::Expr) -> syn::Expr> DeserializeBuilder for DeserializeVariantBuilder<'_, T> {
    fn deserialize_fn_body(
        &self,

        deserializer_ident: &Ident,
        deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError> {
        RecordDeserializeBuilder::new(self.record, self.opts)
            .deserialize_fn_body(deserializer_ident, deserialize_lifetime)
    }

    fn ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(self.record.impl_for_ident.as_ref())
    }

    fn generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(self.record.generics.as_ref())
    }
}
