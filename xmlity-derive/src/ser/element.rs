use quote::{quote, ToTokens};
use syn::{parse_quote, Arm, DataEnum, DataStruct, DeriveInput, Generics, Ident, ItemImpl, Stmt};

use crate::options::{XmlityRootElementDeriveOpts, XmlityRootValueDeriveOpts};
use crate::{
    DeriveError, DeriveMacro, FieldIdent, SerializeField, XmlityFieldAttributeGroupDeriveOpts,
    XmlityFieldDeriveOpts, XmlityFieldElementGroupDeriveOpts,
};

use crate::ExpandedName;

fn derive_fields_struct_serialize(
    expanded_name: &ExpandedName,
    preferred_prefix: Option<&str>,
    enforce_prefix: bool,
    element_fields: impl IntoIterator<Item = SerializeField<XmlityFieldElementGroupDeriveOpts>>,
    attribute_fields: impl IntoIterator<Item = SerializeField<XmlityFieldAttributeGroupDeriveOpts>>,
) -> Vec<Stmt> {
    let element_access_ident = Ident::new("__element", proc_macro2::Span::call_site());
    let children_access_ident = Ident::new("__children", proc_macro2::Span::call_site());
    let xml_name_temp_ident = Ident::new("__xml_name", proc_macro2::Span::call_site());

    let attribute_fields =
        super::attribute_field_serializer(quote! {#element_access_ident}, attribute_fields);

    let element_fields =
        super::element_field_serializer(quote! {#children_access_ident}, element_fields);

    let preferred_prefix_setting = preferred_prefix.map::<Stmt, _>(|preferred_prefix| parse_quote! {
            ::xmlity::ser::SerializeElement::preferred_prefix(&mut #element_access_ident, ::core::option::Option::Some(::xmlity::Prefix::new(#preferred_prefix).expect("XML prefix in derive macro is invalid. This is a bug in xmlity. Please report it.")))?;
        });
    let enforce_prefix_setting = Some(enforce_prefix).filter(|&enforce_prefix| enforce_prefix).map::<Stmt, _>(|enforce_prefix| parse_quote! {
            ::xmlity::ser::SerializeElement::include_prefix(&mut #element_access_ident, #enforce_prefix)?;
        });

    parse_quote! {
        let #xml_name_temp_ident = #expanded_name;
        let mut #element_access_ident = ::xmlity::Serializer::serialize_element(serializer, &#xml_name_temp_ident)?;
        #preferred_prefix_setting
        #enforce_prefix_setting
        #attribute_fields
        let mut #children_access_ident = ::xmlity::ser::SerializeElement::serialize_children(#element_access_ident)?;
        #element_fields
        ::xmlity::ser::SerializeElementChildren::end(#children_access_ident)
    }
}

fn derive_unit_struct_serialize(expanded_name: &ExpandedName) -> proc_macro2::TokenStream {
    let xml_name_temp_ident = Ident::new("__xml_name", proc_macro2::Span::call_site());

    parse_quote! {
        let #xml_name_temp_ident = #expanded_name;
        ::xmlity::Serializer::serialize_element_empty(serializer, &#xml_name_temp_ident)?;
    }
}

fn derive_enum_serialize(
    ident: &syn::Ident,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
    _element_opts: Option<&crate::XmlityRootElementDeriveOpts>,
    value_opts: Option<&crate::XmlityRootValueDeriveOpts>,
) -> Result<Vec<Stmt>, DeriveError> {
    let variants = variants
        .iter()
        .map::<Result<Arm, DeriveError>, _>(|variant| {
            let variant_ident = &variant.ident;
            let variant_ident_string = value_opts
                .as_ref()
                .map(|a| a.rename_all)
                .unwrap_or_default()
                .apply_to_variant(&variant_ident.to_string());

            match &variant.fields {
                syn::Fields::Named(_fields) => Err(DeriveError::Custom(
                    "Named fields are not supported yet".to_string(),
                )),
                syn::Fields::Unnamed(fields) if fields.unnamed.is_empty() => Ok(parse_quote! {
                    #ident::#variant_ident() => {
                        ::xmlity::Serialize::serialize(&__v, serializer)
                    },
                }),
                syn::Fields::Unnamed(fields) if fields.unnamed.len() != 1 => {
                    Err(DeriveError::Custom(
                        "Enum variants with more than one field are not supported".to_string(),
                    ))
                }
                syn::Fields::Unnamed(_) => Ok(parse_quote! {
                    #ident::#variant_ident(__v) => {
                        ::xmlity::Serialize::serialize(&__v, serializer)
                    },
                }),
                syn::Fields::Unit => Ok(parse_quote! {
                    #ident::#variant_ident => {
                        ::xmlity::Serialize::serialize(&#variant_ident_string, serializer)
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

fn serialize_trait_impl(
    ident: &proc_macro2::Ident,
    generics: &Generics,
    implementation: proc_macro2::TokenStream,
) -> ItemImpl {
    let non_bound_generics = crate::non_bound_generics(generics);

    parse_quote! {
        impl #generics ::xmlity::Serialize for #ident #non_bound_generics {
            fn serialize<S>(&self, mut serializer: S) -> Result<<S as ::xmlity::Serializer>::Ok, <S as ::xmlity::Serializer>::Error>
            where
                S: ::xmlity::Serializer,
            {
                #implementation
            }
        }
    }
}

pub struct DeriveSerialize;

impl DeriveMacro for DeriveSerialize {
    fn input_to_derive(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        let element_opts = XmlityRootElementDeriveOpts::parse(ast)?;
        let value_opts = XmlityRootValueDeriveOpts::parse(ast)?;

        let ident_name = ast.ident.to_string();
        let expanded_name = ExpandedName::new(
            element_opts
                .as_ref()
                .and_then(|o| o.name.0.as_ref())
                .unwrap_or(&ident_name),
            element_opts.as_ref().and_then(|o| o.namespace.0.as_deref()),
        );

        let preferred_prefix = element_opts
            .as_ref()
            .and_then(|o| o.preferred_prefix.0.as_deref());

        let implementation = match &ast.data {
            syn::Data::Struct(DataStruct { fields, .. }) => {
                let fields = match fields {
                    syn::Fields::Named(fields) => fields
                        .named
                        .iter()
                        .map(|f| {
                            darling::Result::Ok(SerializeField {
                                field_ident: FieldIdent::Named(
                                    f.ident.clone().expect("Named struct"),
                                ),
                                options: XmlityFieldDeriveOpts::from_field(f)?,
                                field_type: f.ty.clone(),
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    syn::Fields::Unnamed(fields) => fields
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(i, f)| {
                            darling::Result::Ok(SerializeField {
                                field_ident: FieldIdent::Indexed(syn::Index::from(i)),
                                options: XmlityFieldDeriveOpts::from_field(f)?,
                                field_type: f.ty.clone(),
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    syn::Fields::Unit => return Ok(derive_unit_struct_serialize(&expanded_name)),
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

                derive_fields_struct_serialize(
                    &expanded_name,
                    preferred_prefix,
                    element_opts
                        .as_ref()
                        .map(|o| o.enforce_prefix)
                        .unwrap_or(false),
                    element_group_fields,
                    attribute_group_fields,
                )
            }
            syn::Data::Enum(DataEnum { variants, .. }) => derive_enum_serialize(
                &ast.ident,
                variants,
                element_opts.as_ref(),
                value_opts.as_ref(),
            )?,
            syn::Data::Union(_) => unreachable!(),
        };

        Ok(serialize_trait_impl(
            &ast.ident,
            &ast.generics,
            quote! {
                #(#implementation)*
            },
        )
        .to_token_stream())
    }
}
