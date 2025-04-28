use quote::quote;
use syn::{parse_quote, Data, DataStruct, Ident, Stmt};

use crate::options::{structs::roots::RootElementOpts, WithExpandedNameExt};
use crate::DeriveError;

use super::SerializeBuilder;

pub struct DeriveElementStruct<'a> {
    opts: &'a RootElementOpts,
}

impl<'a> DeriveElementStruct<'a> {
    pub fn new(opts: &'a RootElementOpts) -> Self {
        Self { opts }
    }
}

impl SerializeBuilder for DeriveElementStruct<'_> {
    fn serialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        serializer_access: &Ident,
        _serializer_type: &syn::Type,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let RootElementOpts {
            preferred_prefix,
            enforce_prefix,
            ..
        } = self.opts;

        let ident_name = ast.ident.to_string();
        let expanded_name = self.opts.expanded_name(&ident_name);

        let element_access_ident = Ident::new("__element", proc_macro2::Span::call_site());
        let children_access_ident = Ident::new("__children", proc_macro2::Span::call_site());
        let xml_name_temp_ident = Ident::new("__xml_name", proc_macro2::Span::call_site());

        let Data::Struct(DataStruct { fields, .. }) = &ast.data else {
            unreachable!()
        };

        match fields {
            syn::Fields::Named(_) | syn::Fields::Unnamed(_) => {
                let attribute_fields = crate::ser::attribute_group_field_serializer(
                    quote! {#element_access_ident},
                    crate::ser::attribute_group_fields(ast)?,
                );

                let element_fields = crate::ser::element_group_field_serializer(
                    quote! {#children_access_ident},
                    crate::ser::element_group_fields(ast)?,
                );

                let preferred_prefix_setting = preferred_prefix.as_ref().map::<Stmt, _>(|preferred_prefix| parse_quote! {
                      ::xmlity::ser::SerializeElement::preferred_prefix(&mut #element_access_ident, ::core::option::Option::Some(#preferred_prefix))?;
                  });
                let enforce_prefix_setting = Some(*enforce_prefix).filter(|&enforce_prefix| enforce_prefix).map::<Stmt, _>(|enforce_prefix| parse_quote! {
                      ::xmlity::ser::SerializeElement::include_prefix(&mut #element_access_ident, #enforce_prefix)?;
                  });

                Ok(parse_quote! {
                    let #xml_name_temp_ident = #expanded_name;
                    let mut #element_access_ident = ::xmlity::Serializer::serialize_element(#serializer_access, &#xml_name_temp_ident)?;
                    #preferred_prefix_setting
                    #enforce_prefix_setting
                    #attribute_fields
                    let mut #children_access_ident = ::xmlity::ser::SerializeElement::serialize_children(#element_access_ident)?;
                    #element_fields
                    ::xmlity::ser::SerializeElementChildren::end(#children_access_ident)
                })
            }
            syn::Fields::Unit => {
                let xml_name_temp_ident = Ident::new("__xml_name", proc_macro2::Span::call_site());

                Ok(parse_quote! {
                    let #xml_name_temp_ident = #expanded_name;
                    ::xmlity::Serializer::serialize_element_empty(serializer, &#xml_name_temp_ident)?;
                })
            }
        }
    }
}
