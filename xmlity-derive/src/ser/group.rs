use quote::{quote, ToTokens};
use syn::{parse_quote, DataEnum, DataStruct, DeriveInput, Ident, ImplItemFn, ItemImpl, Stmt};

use crate::{options::XmlityRootGroupDeriveOpts, simple_compile_error, DeriveError, DeriveMacro};

trait SerializationGroupBuilder {
    fn serialize_attributes_fn_body(
        &self,
        ast: &syn::DeriveInput,
        element_access_ident: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError>;

    fn serialize_children_fn_body(
        &self,
        ast: &syn::DeriveInput,
        children_access_ident: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError>;
}

trait SerializationGroupBuilderExt: SerializationGroupBuilder {
    fn serialize_attributes_fn(&self, ast: &syn::DeriveInput) -> Result<ImplItemFn, DeriveError>;
    fn serialize_children_fn(&self, ast: &syn::DeriveInput) -> Result<ImplItemFn, DeriveError>;
    fn serialize_trait_impl(&self, ast: &syn::DeriveInput) -> Result<ItemImpl, DeriveError>;
}

impl<T: SerializationGroupBuilder> SerializationGroupBuilderExt for T {
    fn serialize_attributes_fn(&self, ast: &syn::DeriveInput) -> Result<ImplItemFn, DeriveError> {
        let element_access_ident = Ident::new("__element", proc_macro2::Span::call_site());
        let body = self.serialize_attributes_fn_body(ast, &element_access_ident)?;

        Ok(parse_quote!(
            fn serialize_attributes<S: xmlity::ser::SerializeAttributes>(
                &self,
                mut #element_access_ident: S,
            ) -> Result<(), <S as xmlity::ser::SerializeAttributes>::Error> {
               #(#body)*
            }
        ))
    }

    fn serialize_children_fn(&self, ast: &syn::DeriveInput) -> Result<ImplItemFn, DeriveError> {
        let children_access_ident = Ident::new("__children", proc_macro2::Span::call_site());
        let body = self.serialize_children_fn_body(ast, &children_access_ident)?;

        Ok(parse_quote!(
            fn serialize_children<S: xmlity::ser::SerializeChildren>(
                &self,
                mut #children_access_ident: S,
            ) -> Result<(), <S as xmlity::ser::SerializeChildren>::Error> {
                #(#body)*
            }
        ))
    }

    fn serialize_trait_impl(&self, ast: &syn::DeriveInput) -> Result<ItemImpl, DeriveError> {
        let DeriveInput {
            ident, generics, ..
        } = ast;
        let non_bound_generics = crate::non_bound_generics(generics);

        let serialize_attributes_fn = self.serialize_attributes_fn(ast)?;
        let serialize_children_fn = self.serialize_children_fn(ast)?;

        Ok(parse_quote! {
        impl #generics ::xmlity::ser::SerializationGroup for #ident #non_bound_generics {
            #serialize_attributes_fn

            #serialize_children_fn
        }
        })
    }
}

#[allow(unused)]
pub struct DeriveGroupStruct<'a> {
    opts: &'a XmlityRootGroupDeriveOpts,
}

impl<'a> DeriveGroupStruct<'a> {
    fn new(opts: &'a XmlityRootGroupDeriveOpts) -> Self {
        Self { opts }
    }
}

impl SerializationGroupBuilder for DeriveGroupStruct<'_> {
    fn serialize_attributes_fn_body(
        &self,
        ast: &syn::DeriveInput,
        element_access_ident: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let serialize_attributes_implementation = super::attribute_group_field_serializer(
            quote! {&mut #element_access_ident},
            crate::ser::attribute_group_fields(ast)?,
        );

        Ok(parse_quote! {
            #serialize_attributes_implementation
            ::core::result::Result::Ok(())
        })
    }

    fn serialize_children_fn_body(
        &self,
        ast: &syn::DeriveInput,
        children_access_ident: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let serialize_children_implementation = super::element_group_field_serializer(
            quote! {&mut #children_access_ident},
            crate::ser::element_group_fields(ast)?,
        );

        Ok(parse_quote! {
            #serialize_children_implementation
            ::core::result::Result::Ok(())
        })
    }
}

pub struct DeriveSerializationGroup;

impl DeriveMacro for DeriveSerializationGroup {
    fn input_to_derive(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        let opts = XmlityRootGroupDeriveOpts::parse(ast)
            .expect("Wrong options")
            .unwrap_or_default();

        match &ast.data {
            syn::Data::Struct(DataStruct { fields, .. }) => {
                if let syn::Fields::Unit = fields {
                    return Ok(simple_compile_error("Unit structs are not supported"));
                };

                DeriveGroupStruct::new(&opts)
                    .serialize_trait_impl(ast)
                    .map(|a| a.to_token_stream())
            }
            syn::Data::Enum(DataEnum { .. }) => {
                Ok(simple_compile_error("Enum is not supported yet."))
            }
            syn::Data::Union(_) => Ok(simple_compile_error("Union is not supported.")),
        }
    }
}
