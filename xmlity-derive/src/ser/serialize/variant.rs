#![allow(clippy::type_complexity)]

use std::borrow::Cow;

use proc_macro2::Span;
use syn::{parse_quote, Expr, Ident, LifetimeParam, Stmt};

use crate::{
    common::RecordInput,
    options::{records, Extendable},
    ser::builders::SerializeBuilder,
    DeriveError,
};

use super::RecordSerializeBuilder;

pub struct SerializeVariantBuilder<'a, T: Fn(syn::Expr) -> syn::Expr> {
    pub record: &'a RecordInput<'a, T>,
    pub opts: &'a records::roots::SerializeRootOpts,
}

impl<'a, T: Fn(syn::Expr) -> syn::Expr> SerializeVariantBuilder<'a, T> {
    pub fn new(record: &'a RecordInput<T>, opts: &'a records::roots::SerializeRootOpts) -> Self {
        Self { record, opts }
    }

    fn value_access_ident(&self) -> Ident {
        Ident::new("__value", Span::call_site())
    }

    fn serialize_lifetime(&self) -> syn::Lifetime {
        parse_quote!('__xmlity)
    }

    fn generics_with_serialize_lifetime(&self) -> syn::Generics {
        let mut generics = self.generics().into_owned();
        let lifetime = self.serialize_lifetime();
        generics
            .params
            .insert(0, syn::GenericParam::Lifetime(LifetimeParam::new(lifetime)));
        generics
    }

    pub fn definition(&self) -> syn::ItemStruct {
        let Self { record, .. } = self;

        let ident = record.impl_for_ident.as_ref();
        let enum_type = record.result_type.as_ref();

        let value_access_ident = self.value_access_ident();

        let generics = self.generics_with_serialize_lifetime();
        // let generics = self.generics();

        parse_quote! {
            struct #ident #generics {
                #value_access_ident: &'__xmlity #enum_type,
            }
        }
    }

    pub fn serialize_expr(&self, sub_value: &Ident) -> Expr {
        let Self { record, .. } = self;
        let ident = record.impl_for_ident.as_ref();
        let value_access_ident = self.value_access_ident();
        parse_quote!(
            #ident {
                #value_access_ident: &#sub_value,
            }
        )
    }
}

impl<T: Fn(syn::Expr) -> syn::Expr> SerializeBuilder for SerializeVariantBuilder<'_, T> {
    fn serialize_fn_body(
        &self,
        serializer_access: &Ident,
        serializer_type: &syn::Type,
    ) -> Result<Vec<Stmt>, DeriveError> {
        RecordSerializeBuilder::new(self.record, self.opts)
            .serialize_fn_body(serializer_access, serializer_type)
    }

    fn ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(self.record.impl_for_ident.as_ref())
    }

    fn generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(self.record.generics.as_ref())
    }
}
