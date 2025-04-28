mod none;
pub use none::{DeriveEnum, DeriveNoneStruct};
mod element;
pub use element::DeriveElementStruct;

use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{parse_quote, DeriveInput, Ident, ImplItemFn, ItemImpl, Stmt};

use crate::options::{XmlityRootElementDeriveOpts, XmlityRootValueDeriveOpts};
use crate::{DeriveError, DeriveMacro};

trait SerializeBuilder {
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

enum DeriveSerializeOption {
    Element(XmlityRootElementDeriveOpts),
    Value(XmlityRootValueDeriveOpts),
    None,
}

impl DeriveSerializeOption {
    pub fn parse(ast: &DeriveInput) -> Result<Self, DeriveError> {
        let element_opts = XmlityRootElementDeriveOpts::parse(ast)?;
        let value_opts = XmlityRootValueDeriveOpts::parse(ast)?;

        match (element_opts, value_opts) {
            (Some(element_opts), None) => Ok(DeriveSerializeOption::Element(element_opts)),
            (None, Some(value_opts)) => Ok(DeriveSerializeOption::Value(value_opts)),
            (None, None) => Ok(DeriveSerializeOption::None),
            _ => Err(DeriveError::custom(
                "Wrong options. Only one of `xelement` or `xvalue` can be used for root elements.",
            )),
        }
    }
}

pub struct DeriveSerialize;

impl DeriveMacro for DeriveSerialize {
    fn input_to_derive(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        let opts = DeriveSerializeOption::parse(ast)?;

        match (&ast.data, &opts) {
            // `xelement`
            (syn::Data::Struct(_), DeriveSerializeOption::Element(opts)) => {
                DeriveElementStruct::new(opts)
                    .serialize_trait_impl(ast)
                    .map(|a| a.to_token_stream())
            }
            (syn::Data::Enum(_), DeriveSerializeOption::Element(_)) => Err(DeriveError::custom(
                "`xelement` is not compatible with enums.",
            )),
            // `xvalue`
            (syn::Data::Struct(_), DeriveSerializeOption::Value(_)) => Err(DeriveError::custom(
                "`xvalue` is not compatible with structs.",
            )),
            (syn::Data::Enum(_), DeriveSerializeOption::Value(opts)) => DeriveEnum::new(Some(opts))
                .serialize_trait_impl(ast)
                .map(|a| a.to_token_stream()),
            // None
            (syn::Data::Struct(_), DeriveSerializeOption::None) => DeriveNoneStruct::new()
                .serialize_trait_impl(ast)
                .map(|a| a.to_token_stream()),
            (syn::Data::Enum(_), DeriveSerializeOption::None) => DeriveEnum::new(None)
                .serialize_trait_impl(ast)
                .map(|a| a.to_token_stream()),
            (syn::Data::Union(_), _) => Err(DeriveError::custom(
                "Unions are not supported for serialization.",
            )),
        }
    }
}
