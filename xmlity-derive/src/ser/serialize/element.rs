use std::borrow::Cow;

use quote::quote;
use syn::{parse_quote, Data, DataStruct, Ident, Stmt};

use crate::de::StructTypeWithFields;
use crate::options::structs::fields::FieldOpts;
use crate::options::{structs::roots::RootElementOpts, WithExpandedNameExt};
use crate::options::{FieldWithOpts, Prefix};
use crate::{DeriveError, DeriveResult, ExpandedName, FieldIdent};

use super::SerializeBuilder;

pub struct DeriveElementStruct<'a> {
    opts: &'a RootElementOpts,
    ast: &'a syn::DeriveInput,
}

impl<'a> DeriveElementStruct<'a> {
    pub fn new(opts: &'a RootElementOpts, ast: &'a syn::DeriveInput) -> Self {
        Self { opts, ast }
    }

    pub fn to_builder(&self) -> DeriveResult<StructSerializeElementBuilder> {
        let Data::Struct(DataStruct { fields, .. }) = &self.ast.data else {
            unreachable!()
        };

        #[allow(clippy::type_complexity)]
        let struct_type: StructTypeWithFields<
            Vec<FieldWithOpts<syn::Ident, FieldOpts>>,
            Vec<FieldWithOpts<syn::Index, FieldOpts>>,
        > = match fields {
            syn::Fields::Named(fields) => StructTypeWithFields::Named(
                fields
                    .named
                    .iter()
                    .map(|f| {
                        let field_ident = f.ident.clone().expect("Named struct");

                        DeriveResult::Ok(FieldWithOpts {
                            field_ident,
                            options: FieldOpts::from_field(f)?,
                            field_type: f.ty.clone(),
                        })
                    })
                    .collect::<Result<Vec<_>, DeriveError>>()?,
            ),
            syn::Fields::Unnamed(fields) => StructTypeWithFields::Unnamed(
                fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        DeriveResult::Ok(FieldWithOpts {
                            field_ident: syn::Index::from(i),
                            options: FieldOpts::from_field(f)?,
                            field_type: f.ty.clone(),
                        })
                    })
                    .collect::<Result<Vec<_>, DeriveError>>()?,
            ),
            _ => StructTypeWithFields::Unit,
        };

        Ok(StructSerializeElementBuilder {
            ident: &self.ast.ident,
            generics: &self.ast.generics,
            expanded_name: self
                .opts
                .expanded_name(&self.ast.ident.to_string())
                .into_owned(),
            struct_type,
            preferred_prefix: self.opts.preferred_prefix.clone(),
            enforce_prefix: self.opts.enforce_prefix,
        })
    }
}

#[allow(clippy::type_complexity)]
pub struct StructSerializeElementBuilder<'a> {
    pub ident: &'a syn::Ident,
    pub generics: &'a syn::Generics,
    pub expanded_name: ExpandedName<'static>,
    pub struct_type: StructTypeWithFields<
        Vec<FieldWithOpts<syn::Ident, FieldOpts>>,
        Vec<FieldWithOpts<syn::Index, FieldOpts>>,
    >,
    pub preferred_prefix: Option<Prefix<'static>>,
    pub enforce_prefix: bool,
}

impl SerializeBuilder for StructSerializeElementBuilder<'_> {
    fn serialize_fn_body(
        &self,
        serializer_access: &Ident,
        _serializer_type: &syn::Type,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let Self {
            expanded_name,
            preferred_prefix,
            enforce_prefix,
            struct_type,
            ..
        } = self;

        let element_access_ident = Ident::new("__element", proc_macro2::Span::call_site());
        let children_access_ident = Ident::new("__children", proc_macro2::Span::call_site());
        let xml_name_temp_ident = Ident::new("__xml_name", proc_macro2::Span::call_site());

        let fields = match struct_type {
            StructTypeWithFields::Named(fields) => fields
                .iter()
                .cloned()
                .map(|a| a.map_ident(FieldIdent::Named))
                .collect::<Vec<_>>(),
            StructTypeWithFields::Unnamed(fields) => fields
                .iter()
                .cloned()
                .map(|a| a.map_ident(FieldIdent::Indexed))
                .collect::<Vec<_>>(),
            StructTypeWithFields::Unit => vec![],
        };
        let attribute_fields = crate::ser::attribute_group_fields(fields.clone())?;
        let element_fields = crate::ser::element_group_fields(fields)?;

        let attribute_fields = crate::ser::attribute_group_field_serializer(
            quote! {#element_access_ident},
            attribute_fields,
        );

        let element_end = if element_fields.is_empty() {
            quote! {
                ::xmlity::ser::SerializeElement::end(#element_access_ident)
            }
        } else {
            let element_fields = crate::ser::element_group_field_serializer(
                quote! {#children_access_ident},
                element_fields,
            );

            quote! {
                let mut #children_access_ident = ::xmlity::ser::SerializeElement::serialize_children(#element_access_ident)?;
                #element_fields
                ::xmlity::ser::SerializeElementChildren::end(#children_access_ident)
            }
        };

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
            #element_end
        })
    }

    fn ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(self.ident)
    }

    fn generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(self.generics)
    }
}
