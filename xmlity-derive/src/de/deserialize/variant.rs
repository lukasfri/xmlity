#![allow(clippy::type_complexity)]

use std::borrow::Cow;

use syn::{parse_quote, Ident, Lifetime, Stmt};

use crate::{
    de::builders::DeserializeBuilder,
    options::{enums, records},
    DeriveError,
};

use super::{RecordDeserializeBuilder, RecordInput};

pub struct DeserializeVariantBuilder<'a, T: Fn(syn::Expr) -> syn::Expr> {
    pub record: &'a RecordInput<'a, T>,
    pub opts: &'a enums::variants::DeserializeRootOpts,
}

impl<'a, T: Fn(syn::Expr) -> syn::Expr> DeserializeVariantBuilder<'a, T> {
    pub fn new(record: &'a RecordInput<T>, opts: &'a enums::variants::DeserializeRootOpts) -> Self {
        Self { record, opts }
    }

    pub fn value_access_ident(&self) -> Ident {
        self.record
            .sub_path_ident
            .clone()
            .expect("This should be set for variants")
    }

    pub fn definition(&self) -> syn::ItemStruct {
        let Self { record, .. } = self;

        let ident = record.impl_for_ident.as_ref();
        let generics = record.generics.as_ref();
        let enum_type = record.result_type.as_ref();

        let value_access_ident = self.value_access_ident();

        parse_quote! {
            #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
            struct #ident #generics {
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
        RecordDeserializeBuilder::new(
            self.record,
            &match self.opts {
                enums::variants::DeserializeRootOpts::None => {
                    records::roots::DeserializeRootOpts::None
                }
                enums::variants::DeserializeRootOpts::Element(opts) => {
                    records::roots::DeserializeRootOpts::Element(opts.clone())
                }
                enums::variants::DeserializeRootOpts::Attribute(opts) => {
                    records::roots::DeserializeRootOpts::Attribute(opts.clone())
                }
                enums::variants::DeserializeRootOpts::Value(opts) => {
                    let opts = opts.clone();
                    records::roots::DeserializeRootOpts::Value(records::roots::RootValueOpts {
                        value: opts.value,
                        ignore_whitespace: opts.ignore_whitespace,
                        ignore_comments: opts.ignore_comments,
                        allow_unknown: opts.allow_unknown,
                        order: opts.order,
                        with: None,
                        serialize_with: None,
                        deserialize_with: None,
                    })
                }
            },
        )
        .deserialize_fn_body(deserializer_ident, deserialize_lifetime)
    }

    fn ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(self.record.impl_for_ident.as_ref())
    }

    fn generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(self.record.generics.as_ref())
    }
}
