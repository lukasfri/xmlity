use quote::quote;
use syn::{DataEnum, DataStruct, DeriveInput, Ident};

use crate::{
    simple_compile_error, FieldIdent, SerializeField, XmlityFieldAttributeGroupDeriveOpts,
    XmlityFieldDeriveOpts, XmlityFieldElementGroupDeriveOpts,
};

fn deserialize_trait_impl(
    ident: &proc_macro2::Ident,
    element_access_ident: &proc_macro2::Ident,
    serialize_attributes_implementation: proc_macro2::TokenStream,
    children_access_ident: &proc_macro2::Ident,
    serialize_children_implementation: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
    impl ::xmlity::ser::SerializationGroup for #ident {
        fn serialize_attributes<S: xmlity::ser::SerializeAttributes>(
            &self,
            mut #element_access_ident: S,
        ) -> Result<(), <S as xmlity::ser::SerializeAttributes>::Error> {
           #serialize_attributes_implementation
        }

        fn serialize_children<S: xmlity::ser::SerializeChildren>(
            &self,
            mut #children_access_ident: S,
        ) -> Result<(), <S as xmlity::ser::SerializeChildren>::Error> {
           #serialize_children_implementation
        }
    }
    }
}

pub fn derive_serialize_fn(
    ast: DeriveInput,
    _opts: crate::XmlityRootGroupDeriveOpts,
) -> darling::Result<proc_macro2::TokenStream> {
    match ast.data {
        syn::Data::Struct(DataStruct { fields, .. }) => {
            let fields = match fields {
                syn::Fields::Named(fields) => fields
                    .named
                    .into_iter()
                    .map(|f| {
                        darling::Result::Ok(SerializeField {
                            field_ident: FieldIdent::Named(f.ident.clone().expect("Named struct")),
                            options: XmlityFieldDeriveOpts::from_field(&f)?,
                            field_type: f.ty,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                syn::Fields::Unnamed(fields) => fields
                    .unnamed
                    .into_iter()
                    .enumerate()
                    .map(|(i, f)| {
                        darling::Result::Ok(SerializeField {
                            field_ident: FieldIdent::Indexed(syn::Index::from(i)),
                            options: XmlityFieldDeriveOpts::from_field(&f)?,
                            field_type: f.ty,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                syn::Fields::Unit => {
                    return Ok(simple_compile_error("Unit structs are not supported"))
                }
            };

            let attribute_group_fields = fields.clone().into_iter().filter_map(|field| {
                field.map_options_opt(|opt| match opt {
                    XmlityFieldDeriveOpts::Attribute(opts) => {
                        Some(XmlityFieldAttributeGroupDeriveOpts::Attribute(opts))
                    }
                    XmlityFieldDeriveOpts::Group(opts) => {
                        Some(XmlityFieldAttributeGroupDeriveOpts::Group(opts))
                    }
                    XmlityFieldDeriveOpts::Element(_) => None,
                })
            });

            let element_group_fields = fields.clone().into_iter().filter_map(|field| {
                field.map_options_opt(|opt| match opt {
                    XmlityFieldDeriveOpts::Element(opts) => {
                        Some(XmlityFieldElementGroupDeriveOpts::Element(opts))
                    }
                    XmlityFieldDeriveOpts::Group(opts) => {
                        Some(XmlityFieldElementGroupDeriveOpts::Group(opts))
                    }
                    XmlityFieldDeriveOpts::Attribute(_) => None,
                })
            });

            let element_access_ident = Ident::new("__element", proc_macro2::Span::call_site());
            let children_access_ident = Ident::new("__children", proc_macro2::Span::call_site());
            let serialize_attributes_implementation = super::attribute_field_serializer(
                quote! {&mut #element_access_ident},
                attribute_group_fields,
            );

            let serialize_attributes_implementation = quote! {
                #serialize_attributes_implementation
                ::core::result::Result::Ok(())
            };

            let serialize_children_implementation = super::element_field_serializer(
                quote! {&mut #children_access_ident},
                element_group_fields,
            );

            let serialize_children_implementation = quote! {
                #serialize_children_implementation
                ::core::result::Result::Ok(())
            };

            Ok(deserialize_trait_impl(
                &ast.ident,
                &element_access_ident,
                serialize_attributes_implementation,
                &children_access_ident,
                serialize_children_implementation,
            ))
        }
        syn::Data::Enum(DataEnum { .. }) => Ok(simple_compile_error("Enum is not supported yet.")),
        syn::Data::Union(_) => Ok(simple_compile_error("Union is not supported.")),
    }
}
