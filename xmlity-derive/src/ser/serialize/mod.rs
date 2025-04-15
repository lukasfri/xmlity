mod none;
pub use none::{DeriveNoneEnum, DeriveNoneStruct};
mod value;
pub use value::DeriveValueEnum;
mod element;
pub use element::DeriveElementStruct;

use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{parse_quote, DeriveInput, Ident, ImplItemFn, ItemImpl, Stmt};

use crate::options::{XmlityRootElementDeriveOpts, XmlityRootValueDeriveOpts};
use crate::{DeriveError, DeriveMacro};

trait SerializeBuilder {
    /// Returns the content inside the `Deserialize::deserialize` function.
    fn serialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        serializer_access: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError>;
}

trait SerializeBuilderExt: SerializeBuilder {
    fn serialize_fn(&self, ast: &syn::DeriveInput) -> Result<ImplItemFn, DeriveError>;
    fn serialize_trait_impl(&self, ast: &syn::DeriveInput) -> Result<ItemImpl, DeriveError>;
}

impl<T: SerializeBuilder> SerializeBuilderExt for T {
    fn serialize_fn(&self, ast: &syn::DeriveInput) -> Result<ImplItemFn, DeriveError> {
        let serializer_access_ident = Ident::new("__serializer", ast.span());
        let body = self.serialize_fn_body(ast, &serializer_access_ident)?;
        Ok(parse_quote!(
            fn serialize<S>(&self, mut #serializer_access_ident: S) -> Result<<S as ::xmlity::Serializer>::Ok, <S as ::xmlity::Serializer>::Error>
            where
                S: ::xmlity::Serializer,
            {
                #(#body)*
            }
        ))
    }

    fn serialize_trait_impl(
        &self,
        ast @ DeriveInput {
            ident, generics, ..
        }: &syn::DeriveInput,
    ) -> Result<ItemImpl, DeriveError> {
        let serialize_fn = self.serialize_fn(ast)?;

        let non_bound_generics = crate::non_bound_generics(generics);

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
        let element_opts = XmlityRootElementDeriveOpts::parse(ast)?;
        let value_opts = XmlityRootValueDeriveOpts::parse(ast)?;

        match &ast.data {
            syn::Data::Struct(_) => match element_opts {
                Some(opts) => DeriveElementStruct::new(&opts)
                    .serialize_trait_impl(ast)
                    .map(|a| a.to_token_stream()),
                None => DeriveNoneStruct::new()
                    .serialize_trait_impl(ast)
                    .map(|a| a.to_token_stream()),
            },
            syn::Data::Enum(_) => {
                if let Some(value_opts) = value_opts.as_ref() {
                    DeriveValueEnum::new(value_opts)
                        .serialize_trait_impl(ast)
                        .map(|a| a.to_token_stream())
                } else {
                    DeriveNoneEnum::new()
                        .serialize_trait_impl(ast)
                        .map(|a| a.to_token_stream())
                }
            }
            syn::Data::Union(_) => unreachable!(),
        }
    }
}
