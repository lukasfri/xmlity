use quote::{quote, ToTokens};
use syn::{parse_quote, DataEnum, DataStruct, DeriveInput, Ident, ImplItemFn, ItemImpl, Stmt};

use crate::{
    options::XmlityRootGroupDeriveOpts, simple_compile_error, DeriveError, DeriveMacro, FieldIdent,
    SerializeField, XmlityFieldAttributeGroupDeriveOpts, XmlityFieldDeriveOpts,
    XmlityFieldElementGroupDeriveOpts,
};

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

pub struct DeriveGroupStruct<'a> {
    opts: &'a XmlityRootGroupDeriveOpts,
}

impl<'a> DeriveGroupStruct<'a> {
    fn new(opts: &'a XmlityRootGroupDeriveOpts) -> Self {
        Self { opts }
    }

    fn fields(
        ast: &syn::DeriveInput,
    ) -> Result<Vec<SerializeField<XmlityFieldDeriveOpts>>, DeriveError> {
        let syn::Data::Struct(DataStruct { fields, .. }) = &ast.data else {
            unreachable!()
        };

        match fields {
            syn::Fields::Named(fields) => fields
                .named
                .iter()
                .map(|f| {
                    Ok(SerializeField {
                        field_ident: FieldIdent::Named(f.ident.clone().expect("Named struct")),
                        options: XmlityFieldDeriveOpts::from_field(f)?,
                        field_type: f.ty.clone(),
                    })
                })
                .collect::<Result<Vec<_>, _>>(),
            syn::Fields::Unnamed(fields) => fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    Ok(SerializeField {
                        field_ident: FieldIdent::Indexed(syn::Index::from(i)),
                        options: XmlityFieldDeriveOpts::from_field(f)?,
                        field_type: f.ty.clone(),
                    })
                })
                .collect::<Result<Vec<_>, _>>(),
            syn::Fields::Unit => unreachable!(),
        }
    }

    fn attribute_group_fields(
        ast: &syn::DeriveInput,
    ) -> Result<Vec<SerializeField<XmlityFieldAttributeGroupDeriveOpts>>, DeriveError> {
        Ok(Self::fields(ast)?
            .into_iter()
            .filter_map(|field| {
                field.map_options_opt(|opt| match opt {
                    XmlityFieldDeriveOpts::Attribute(opts) => {
                        Some(XmlityFieldAttributeGroupDeriveOpts::Attribute(opts))
                    }
                    XmlityFieldDeriveOpts::Group(opts) => {
                        Some(XmlityFieldAttributeGroupDeriveOpts::Group(opts))
                    }
                    XmlityFieldDeriveOpts::Element(_) => None,
                })
            })
            .collect())
    }

    fn element_group_fields(
        ast: &syn::DeriveInput,
    ) -> Result<Vec<SerializeField<XmlityFieldElementGroupDeriveOpts>>, DeriveError> {
        Ok(Self::fields(ast)?
            .into_iter()
            .filter_map(|field| {
                field.map_options_opt(|opt| match opt {
                    XmlityFieldDeriveOpts::Element(opts) => {
                        Some(XmlityFieldElementGroupDeriveOpts::Element(opts))
                    }
                    XmlityFieldDeriveOpts::Group(opts) => {
                        Some(XmlityFieldElementGroupDeriveOpts::Group(opts))
                    }
                    XmlityFieldDeriveOpts::Attribute(_) => None,
                })
            })
            .collect())
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
            Self::attribute_group_fields(ast)?,
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
            Self::element_group_fields(ast)?,
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
