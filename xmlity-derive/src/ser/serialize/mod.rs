mod none;

use std::borrow::Cow;

pub use none::{DeriveEnum, RecordSerializeValueBuilder};
mod element;
pub use element::{RecordSerializeElementBuilder, SingleChildSerializeElementBuilder};
mod variant;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, DeriveInput, Ident, Index, Stmt};

use crate::common::{self, FieldIdent, RecordInput, StructTypeWithFields};
use crate::options::records::fields::FieldOpts;
use crate::options::{enums, records, FieldWithOpts};
use crate::{DeriveError, DeriveMacro};

use super::builders::{SerializeBuilder, SerializeBuilderExt};

#[allow(clippy::type_complexity)]
fn value_deconstructor(
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

pub struct RecordSerializeBuilder<'a, T: Fn(syn::Expr) -> syn::Expr> {
    pub input: &'a RecordInput<'a, T>,
    pub options: &'a records::roots::SerializeRootOpts,
}

impl<'a, T: Fn(syn::Expr) -> syn::Expr> RecordSerializeBuilder<'a, T> {
    pub fn new(
        input: &'a RecordInput<'a, T>,
        options: &'a records::roots::SerializeRootOpts,
    ) -> Self {
        Self { input, options }
    }
}

impl<T: Fn(syn::Expr) -> syn::Expr> SerializeBuilder for RecordSerializeBuilder<'_, T> {
    fn serialize_fn_body(
        &self,
        serializer_access: &Ident,
        serializer_type: &syn::Type,
    ) -> Result<Vec<syn::Stmt>, DeriveError> {
        use records::roots::SerializeRootOpts;
        match &self.options {
            SerializeRootOpts::Element(opts) => {
                RecordSerializeElementBuilder::new(self.input, opts)
                    .serialize_fn_body(serializer_access, serializer_type)
            }
            SerializeRootOpts::Value(opts) => {
                RecordSerializeValueBuilder::new(self.input, Some(opts))
                    .serialize_fn_body(serializer_access, serializer_type)
            }
            SerializeRootOpts::None => RecordSerializeValueBuilder::new(self.input, None)
                .serialize_fn_body(serializer_access, serializer_type),
        }
    }

    fn ident(&self) -> Cow<'_, syn::Ident> {
        Cow::Borrowed(self.input.impl_for_ident.as_ref())
    }

    fn generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(self.input.generics.as_ref())
    }
}

pub struct DeriveSerialize;

impl DeriveMacro for DeriveSerialize {
    fn input_to_derive(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        match &ast.data {
            syn::Data::Struct(_) => {
                let record = common::parse_struct_derive_input(ast)?;
                let opts = records::roots::SerializeRootOpts::parse(&ast.attrs)?;
                match opts {
                    records::roots::SerializeRootOpts::Element(opts) => {
                        RecordSerializeElementBuilder::new(&record, &opts)
                            .serialize_trait_impl()
                            .map(|a| a.to_token_stream())
                    }
                    records::roots::SerializeRootOpts::Value(opts) => {
                        RecordSerializeValueBuilder::new(&record, Some(&opts))
                            .serialize_trait_impl()
                            .map(|a| a.to_token_stream())
                    }
                    records::roots::SerializeRootOpts::None => {
                        RecordSerializeValueBuilder::new(&record, None)
                            .serialize_trait_impl()
                            .map(|a| a.to_token_stream())
                    }
                }
            }
            syn::Data::Enum(_) => {
                let opts = enums::roots::RootOpts::parse(ast)?;

                match opts {
                    enums::roots::RootOpts::Value(opts) => DeriveEnum::new(ast, Some(&opts))
                        .serialize_trait_impl()
                        .map(|a| a.to_token_stream()),
                    enums::roots::RootOpts::None => DeriveEnum::new(ast, None)
                        .serialize_trait_impl()
                        .map(|a| a.to_token_stream()),
                }
            }
            syn::Data::Union(_) => Err(DeriveError::custom(
                "Unions are not supported for serialization.",
            )),
        }
    }
}
