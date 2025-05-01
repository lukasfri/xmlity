use std::borrow::Cow;

use proc_macro2::Span;
use syn::{parse_quote, Generics, Ident, ImplItemFn, ItemImpl, Stmt, Type};

use crate::{common::non_bound_generics, DeriveError};

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

        let non_bound_generics = crate::common::non_bound_generics(&generics);

        Ok(parse_quote! {
            impl #generics ::xmlity::Serialize for #ident #non_bound_generics {
                #serialize_fn
            }
        })
    }
}

pub trait SerializeAttributeBuilder {
    fn serialize_attribute_fn_body(
        &self,
        serializer_access: &Ident,
        serializer_type: &syn::Type,
    ) -> Result<Vec<Stmt>, DeriveError>;

    fn ident(&self) -> Cow<'_, Ident>;
    fn generics(&self) -> Cow<'_, syn::Generics>;
}

pub trait SerializeAttributeBuilderExt: SerializeAttributeBuilder {
    fn serialize_attribute_fn(&self) -> Result<ImplItemFn, DeriveError>;
    fn serialize_attribute_trait_impl(&self) -> Result<ItemImpl, DeriveError>;
}

impl<T: SerializeAttributeBuilder> SerializeAttributeBuilderExt for T {
    fn serialize_attribute_fn(&self) -> Result<ImplItemFn, DeriveError> {
        let serializer_access_ident = Ident::new("__serializer", Span::call_site());
        let serializer_type: syn::Type = parse_quote!(__XmlityAttributeSerializer);
        let body = self.serialize_attribute_fn_body(&serializer_access_ident, &serializer_type)?;
        Ok(parse_quote!(
            fn serialize_attribute<#serializer_type>(&self, mut #serializer_access_ident: #serializer_type) -> Result<<#serializer_type as ::xmlity::AttributeSerializer>::Ok, <#serializer_type as ::xmlity::AttributeSerializer>::Error>
            where
                #serializer_type: ::xmlity::AttributeSerializer,
            {
                #(#body)*
            }
        ))
    }

    fn serialize_attribute_trait_impl(&self) -> Result<ItemImpl, DeriveError> {
        let serialize_attribute_fn = self.serialize_attribute_fn()?;
        let ident = self.ident();
        let generics = self.generics();

        let non_bound_generics = non_bound_generics(&generics);

        Ok(parse_quote! {
            impl #generics ::xmlity::SerializeAttribute for #ident #non_bound_generics {
                #serialize_attribute_fn
            }
        })
    }
}

pub trait SerializationGroupBuilder {
    fn serialize_attributes_fn_body(
        &self,
        element_access_ident: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError>;

    fn serialize_children_fn_body(
        &self,
        children_access_ident: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError>;

    fn ident(&self) -> Cow<'_, Ident>;
    fn generics(&self) -> Cow<'_, Generics>;
}

pub trait SerializationGroupBuilderExt: SerializationGroupBuilder {
    fn serialize_attributes_fn(&self) -> Result<ImplItemFn, DeriveError>;
    fn serialize_children_fn(&self) -> Result<ImplItemFn, DeriveError>;
    fn serialization_group_trait_impl(&self) -> Result<ItemImpl, DeriveError>;
}

impl<T: SerializationGroupBuilder> SerializationGroupBuilderExt for T {
    fn serialize_attributes_fn(&self) -> Result<ImplItemFn, DeriveError> {
        let serialize_attributes_ident = Ident::new("__element", proc_macro2::Span::call_site());
        let serialize_attributes_type: Type = parse_quote!(__XmlitySerializeAttributes);
        let body = self.serialize_attributes_fn_body(&serialize_attributes_ident)?;

        Ok(parse_quote!(
            fn serialize_attributes<#serialize_attributes_type: xmlity::ser::SerializeAttributes>(
                &self,
                #serialize_attributes_ident: &mut #serialize_attributes_type,
            ) -> Result<(), <#serialize_attributes_type as xmlity::ser::SerializeAttributes>::Error> {
               #(#body)*
            }
        ))
    }

    fn serialize_children_fn(&self) -> Result<ImplItemFn, DeriveError> {
        let children_access_ident = Ident::new("__children", proc_macro2::Span::call_site());
        let children_access_type: Type = parse_quote!(__XmlitySerializeSeq);
        let body = self.serialize_children_fn_body(&children_access_ident)?;

        Ok(parse_quote!(
            fn serialize_children<#children_access_type: xmlity::ser::SerializeSeq>(
                &self,
                #children_access_ident: &mut #children_access_type,
            ) -> Result<(), <#children_access_type as xmlity::ser::SerializeSeq>::Error> {
                #(#body)*
            }
        ))
    }

    fn serialization_group_trait_impl(&self) -> Result<ItemImpl, DeriveError> {
        let ident = self.ident();
        let generics = self.generics();
        let non_bound_generics = non_bound_generics(&generics);

        let serialize_attributes_fn = self.serialize_attributes_fn()?;
        let serialize_children_fn = self.serialize_children_fn()?;

        Ok(parse_quote! {
        impl #generics ::xmlity::ser::SerializationGroup for #ident #non_bound_generics {
            #serialize_attributes_fn

            #serialize_children_fn
        }
        })
    }
}
