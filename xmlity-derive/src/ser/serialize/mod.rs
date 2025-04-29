mod none;
use std::borrow::Cow;

pub use none::{DeriveEnum, DeriveNoneStruct};
mod element;
pub use element::{DeriveElementStruct, SingleChildSerializeElementBuilder};

use proc_macro2::Span;
use quote::ToTokens;
use syn::{parse_quote, DeriveInput, Generics, Ident, ImplItemFn, ItemImpl, Stmt};

use crate::options::{enums, structs};
use crate::{DeriveError, DeriveMacro};

pub trait SerializeBuilder {
    fn serialize_fn_body(
        &self,
        serializer_access: &Ident,
        serializer_type: &syn::Type,
    ) -> Result<Vec<Stmt>, DeriveError>;

    fn ident(&self) -> Cow<'_, Ident>;
    fn generics(&self) -> Cow<'_, Generics>;
}

pub trait SerializeBuilderExt: SerializeBuilder {
    fn serialize_fn(&self) -> Result<ImplItemFn, DeriveError>;
    fn serialize_trait_impl(&self) -> Result<ItemImpl, DeriveError>;
}

impl<T: SerializeBuilder> SerializeBuilderExt for T {
    fn serialize_fn(&self) -> Result<ImplItemFn, DeriveError> {
        let serializer_access_ident = Ident::new("__serializer", Span::call_site());
        let serializer_type: syn::Type = parse_quote!(__XmlitySerializer);
        let body = self.serialize_fn_body(&serializer_access_ident, &serializer_type)?;
        Ok(parse_quote!(
            fn serialize<#serializer_type>(&self, mut #serializer_access_ident: #serializer_type) -> Result<<#serializer_type as ::xmlity::Serializer>::Ok, <#serializer_type as ::xmlity::Serializer>::Error>
            where
                #serializer_type: ::xmlity::Serializer,
            {
                #(#body)*
            }
        ))
    }

    fn serialize_trait_impl(&self) -> Result<ItemImpl, DeriveError> {
        let ident = self.ident();
        let generics = self.generics();
        let serialize_fn = self.serialize_fn()?;

        let non_bound_generics = crate::non_bound_generics(&generics);

        Ok(parse_quote! {
            impl #generics ::xmlity::Serialize for #ident #non_bound_generics {
                #serialize_fn
            }
        })
    }
}

pub struct DeriveSerialize;

impl DeriveMacro for DeriveSerialize {
    fn input_to_derive(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        match &ast.data {
            syn::Data::Struct(_) => {
                let opts = structs::roots::SerializeRootOpts::parse(ast)?;
                match opts {
                    structs::roots::SerializeRootOpts::Element(opts) => {
                        DeriveElementStruct::new(&opts, ast)
                            .to_builder()?
                            .serialize_trait_impl()
                            .map(|a| a.to_token_stream())
                    }
                    structs::roots::SerializeRootOpts::None => DeriveNoneStruct::new(ast)
                        .serialize_trait_impl()
                        .map(|a| a.to_token_stream()),
                    structs::roots::SerializeRootOpts::Value(_) => Err(DeriveError::custom(
                        "`xvalue` is not compatible with structs.",
                    )),
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
