use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{
    parse_quote, Arm, Data, DataEnum, DataStruct, DeriveInput, Ident, ImplItemFn, ItemImpl, Stmt,
};

use crate::options::{
    WithExpandedName, WithExpandedNameExt, XmlityRootElementDeriveOpts, XmlityRootValueDeriveOpts,
};
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

struct DeriveElementStruct<'a> {
    opts: &'a XmlityRootElementDeriveOpts,
}

impl<'a> DeriveElementStruct<'a> {
    fn new(opts: &'a XmlityRootElementDeriveOpts) -> Self {
        Self { opts }
    }
}

impl SerializeBuilder for DeriveElementStruct<'_> {
    fn serialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        serializer_access: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let XmlityRootElementDeriveOpts {
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
                let attribute_fields = super::attribute_group_field_serializer(
                    quote! {#element_access_ident},
                    crate::ser::attribute_group_fields(ast)?,
                );

                let element_fields = super::element_group_field_serializer(
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

struct DeriveNoneStruct {}

impl DeriveNoneStruct {
    fn new() -> Self {
        Self {}
    }
}

impl SerializeBuilder for DeriveNoneStruct {
    fn serialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        serializer_access: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let seq_access_ident = Ident::new("__seq_access", proc_macro2::Span::call_site());

        let Data::Struct(DataStruct { fields, .. }) = &ast.data else {
            unreachable!()
        };

        match fields {
            syn::Fields::Named(_) | syn::Fields::Unnamed(_) => {
                let value_fields = super::seq_field_serializer(
                    quote! {#seq_access_ident},
                    crate::ser::element_fields(ast)?,
                );

                Ok(parse_quote! {
                    let mut #seq_access_ident = ::xmlity::Serializer::serialize_seq(#serializer_access)?;
                    #value_fields
                    ::xmlity::ser::SerializeSeq::end(#seq_access_ident)
                })
            }
            syn::Fields::Unit => Ok(parse_quote! {
                ::xmlity::Serializer::serialize_none(serializer)?;
            }),
        }
    }
}

pub struct DeriveNoneEnum;

impl SerializeBuilder for DeriveNoneEnum {
    fn serialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        serializer_access: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let DeriveInput { ident, data, .. } = ast;

        let Data::Enum(DataEnum { variants, .. }) = &data else {
            unreachable!()
        };

        let variants = variants
            .iter()
            .map::<Result<Arm, DeriveError>, _>(|variant| {
                let variant_ident = &variant.ident;

                match &variant.fields {
                    syn::Fields::Named(_fields) => {
                        Err(DeriveError::custom("Named fields are not supported yet"))
                    }
                    syn::Fields::Unnamed(fields) if fields.unnamed.is_empty() => Ok(parse_quote! {
                        #ident::#variant_ident() => {
                            ::xmlity::Serialize::serialize(&__v, #serializer_access)
                        },
                    }),
                    syn::Fields::Unnamed(fields) if fields.unnamed.len() != 1 => {
                        Err(DeriveError::custom(
                            "Enum variants with more than one field are not supported",
                        ))
                    }
                    syn::Fields::Unnamed(_) => Ok(parse_quote! {
                        #ident::#variant_ident(__v) => {
                            ::xmlity::Serialize::serialize(&__v, #serializer_access)
                        },
                    }),
                    syn::Fields::Unit => Err(DeriveError::custom(
                        "Unsupported unit variant in non-value enum.",
                    )),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(parse_quote! {
            match self {
                #(#variants)*
            }
        })
    }
}

pub struct DeriveValueEnum<'a> {
    opts: &'a XmlityRootValueDeriveOpts,
}

impl<'a> DeriveValueEnum<'a> {
    pub fn new(opts: &'a XmlityRootValueDeriveOpts) -> Self {
        Self { opts }
    }
}

impl SerializeBuilder for DeriveValueEnum<'_> {
    fn serialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        serializer_access: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let DeriveInput { ident, data, .. } = ast;

        let Data::Enum(DataEnum { variants, .. }) = &data else {
            unreachable!()
        };

        let variants = variants
            .iter()
            .map::<Result<Arm, DeriveError>, _>(|variant| {
                let variant_ident = &variant.ident;
                let variant_ident_string = self
                    .opts
                    .rename_all
                    .apply_to_variant(&variant_ident.to_string());

                match &variant.fields {
                    syn::Fields::Named(_) | syn::Fields::Unnamed(_) => Err(DeriveError::custom(
                        "Unsupported named/unnamed field variant in value enum.",
                    )),
                    syn::Fields::Unit => Ok(parse_quote! {
                        #ident::#variant_ident => {
                            ::xmlity::Serialize::serialize(&#variant_ident_string, #serializer_access)
                        },
                    }),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(parse_quote! {
            match self {
                #(#variants)*
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
                    DeriveNoneEnum
                        .serialize_trait_impl(ast)
                        .map(|a| a.to_token_stream())
                }
            }
            syn::Data::Union(_) => unreachable!(),
        }
    }
}
