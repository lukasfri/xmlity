use quote::{quote, ToTokens};
use syn::{parse_quote, DataStruct, DeriveInput, Ident, ImplItemFn, ItemImpl, Stmt};

use crate::{options::structs::roots::RootGroupOpts, DeriveError, DeriveMacro};

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
        let serialize_attributes_ident = Ident::new("__element", proc_macro2::Span::call_site());
        let serialize_attributes_type: syn::Type = parse_quote!(__XmlitySerializeAttributes);
        let body = self.serialize_attributes_fn_body(ast, &serialize_attributes_ident)?;

        Ok(parse_quote!(
            fn serialize_attributes<#serialize_attributes_type: xmlity::ser::SerializeAttributes>(
                &self,
                #serialize_attributes_ident: &mut #serialize_attributes_type,
            ) -> Result<(), <#serialize_attributes_type as xmlity::ser::SerializeAttributes>::Error> {
               #(#body)*
            }
        ))
    }

    fn serialize_children_fn(&self, ast: &syn::DeriveInput) -> Result<ImplItemFn, DeriveError> {
        let children_access_ident = Ident::new("__children", proc_macro2::Span::call_site());
        let children_access_type: syn::Type = parse_quote!(__XmlitySerializeSeq);
        let body = self.serialize_children_fn_body(ast, &children_access_ident)?;

        Ok(parse_quote!(
            fn serialize_children<#children_access_type: xmlity::ser::SerializeSeq>(
                &self,
                #children_access_ident: &mut #children_access_type,
            ) -> Result<(), <#children_access_type as xmlity::ser::SerializeSeq>::Error> {
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
pub struct DeriveSerializationGroupStruct<'a> {
    opts: &'a RootGroupOpts,
}

impl<'a> DeriveSerializationGroupStruct<'a> {
    fn new(opts: &'a RootGroupOpts) -> Self {
        Self { opts }
    }
}

impl SerializationGroupBuilder for DeriveSerializationGroupStruct<'_> {
    fn serialize_attributes_fn_body(
        &self,
        ast: &syn::DeriveInput,
        element_access_ident: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let serialize_attributes_implementation = super::attribute_group_field_serializer(
            quote! { #element_access_ident},
            crate::ser::attribute_group_fields(crate::ser::fields(ast)?)?,
        )?;

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
            quote! { #children_access_ident},
            crate::ser::element_group_fields(crate::ser::fields(ast)?)?,
        )?;

        Ok(parse_quote! {
            #serialize_children_implementation
            ::core::result::Result::Ok(())
        })
    }
}

enum SerializationGroupOption {
    Group(RootGroupOpts),
}

impl SerializationGroupOption {
    pub fn parse(ast: &DeriveInput) -> Result<Self, DeriveError> {
        let group_opts = RootGroupOpts::parse(ast)?.unwrap_or_default();

        Ok(SerializationGroupOption::Group(group_opts))
    }
}

pub struct DeriveSerializationGroup;

impl DeriveMacro for DeriveSerializationGroup {
    fn input_to_derive(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        let SerializationGroupOption::Group(opts) = SerializationGroupOption::parse(ast)?;

        match &ast.data {
            syn::Data::Struct(DataStruct {
                fields: syn::Fields::Unit,
                ..
            }) => Err(DeriveError::custom(
                "Unit structs are not supported for serialization groups.",
            )),
            syn::Data::Struct(_) => DeriveSerializationGroupStruct::new(&opts)
                .serialize_trait_impl(ast)
                .map(|a| a.to_token_stream()),
            syn::Data::Enum(_) => Err(DeriveError::custom(
                "Enums are not supported for serialization groups.",
            )),
            syn::Data::Union(_) => Err(DeriveError::custom(
                "Unions are not supported for serialization groups.",
            )),
        }
    }
}
