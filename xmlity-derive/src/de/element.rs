use proc_macro2::Span;
use quote::quote;
use syn::{spanned::Spanned, DataEnum, DataStruct, Field, Ident, Lifetime, Variant};

use crate::{
    de::{
        all_attributes_done, all_elements_done, attribute_done, builder_attribute_field_visitor,
        builder_element_field_visitor, constructor_expr, element_done,
    },
    options::ElementOrder,
    simple_compile_error, DeserializeBuilderField, ExpandedName, FieldIdent,
    XmlityFieldAttributeDeriveOpts, XmlityFieldAttributeGroupDeriveOpts, XmlityFieldDeriveOpts,
    XmlityFieldElementDeriveOpts, XmlityFieldElementGroupDeriveOpts, XmlityFieldGroupDeriveOpts,
};

use super::{common::DeserializeTraitImplBuilder, common::VisitorBuilder, StructType};

fn visit_element_fn_signature(
    body: proc_macro2::TokenStream,
    element_access_ident: &Ident,
    visitor_lifetime: &syn::Lifetime,
) -> proc_macro2::TokenStream {
    quote! {
        fn visit_element<A>(self, mut #element_access_ident: A) -> Result<Self::Value, <A as xmlity::de::AttributesAccess<#visitor_lifetime>>::Error>
        where
            A: xmlity::de::ElementAccess<#visitor_lifetime>,
        {
            #body
        }
    }
}

fn visit_element_data_fn_impl(
    ident: &syn::Ident,
    visitor_lifetime: &syn::Lifetime,
    fields: &syn::Fields,
    children_order: ElementOrder,
    allow_unknown_children: bool,
    attributes_order: ElementOrder,
    allow_unknown_attributes: bool,
) -> Result<proc_macro2::TokenStream, darling::Error> {
    let constructor_type = match fields {
        syn::Fields::Named(_) => StructType::Named,
        syn::Fields::Unnamed(_) => StructType::Unnamed,
        _ => unreachable!(),
    };

    let fields = match fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .map(|f| {
                let field_ident = f.ident.clone().expect("Named struct");

                darling::Result::Ok(DeserializeBuilderField {
                    builder_field_ident: FieldIdent::Named(field_ident.clone()),
                    field_ident: FieldIdent::Named(field_ident),
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
                darling::Result::Ok(DeserializeBuilderField {
                    builder_field_ident: FieldIdent::Named(syn::Ident::new(
                        &format!("__{}", i),
                        f.span(),
                    )),
                    field_ident: FieldIdent::Indexed(syn::Index::from(i)),
                    options: XmlityFieldDeriveOpts::from_field(f)?,
                    field_type: f.ty.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => unreachable!(),
    };

    Ok(struct_fields_visitor_end(
        ident,
        visitor_lifetime,
        fields,
        constructor_type,
        children_order,
        allow_unknown_children,
        attributes_order,
        allow_unknown_attributes,
    ))
}

fn element_visit_fn(
    ident: &syn::Ident,
    visitor_lifetime: &syn::Lifetime,
    expanded_name: Option<ExpandedName>,
    data_struct: &DataStruct,
    children_order: ElementOrder,
    allow_unknown_children: bool,
    attributes_order: ElementOrder,
    allow_unknown_attributes: bool,
) -> Result<proc_macro2::TokenStream, darling::Error> {
    let element_access_ident = syn::Ident::new("__element", ident.span());
    let xml_name_identification = expanded_name.as_ref().map(|qname| {
        quote! {
            ::xmlity::de::ElementAccessExt::ensure_name::<<A as ::xmlity::de::AttributesAccess<#visitor_lifetime>>::Error>(&#element_access_ident, &#qname)?;
        }
    });

    let deserialization_impl = match &data_struct.fields {
        fields @ syn::Fields::Named(_) | fields @ syn::Fields::Unnamed(_) => {
            visit_element_data_fn_impl(
                ident,
                visitor_lifetime,
                fields,
                children_order,
                allow_unknown_children,
                attributes_order,
                allow_unknown_attributes,
            )?
        }
        syn::Fields::Unit => visit_element_unit_fn_impl(ident),
    };

    Ok(visit_element_fn_signature(
        quote! {
            #xml_name_identification

            #deserialization_impl
        },
        &element_access_ident,
        visitor_lifetime,
    ))
}

fn visit_attribute_fn_signature(
    attribute_access_ident: &syn::Ident,
    visitor_lifetime: &syn::Lifetime,
    body: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        fn visit_attribute<A>(self, #attribute_access_ident: A) -> Result<Self::Value, <A as ::xmlity::de::AttributeAccess<#visitor_lifetime>>::Error>
        where
            A: ::xmlity::de::AttributeAccess<#visitor_lifetime>,
        {
            #body
        }
    }
}

fn attribute_visit_fn(
    ident: &syn::Ident,
    visitor_lifetime: &syn::Lifetime,
    expanded_name: Option<ExpandedName>,
    data_struct: &DataStruct,
) -> darling::Result<proc_macro2::TokenStream> {
    let attribute_access_ident = Ident::new("__attribute", ident.span());
    let xml_name_identification = expanded_name.map(|qname| {
        quote! {
            ::xmlity::de::AttributeAccessExt::ensure_name::<<A as ::xmlity::de::AttributeAccess<#visitor_lifetime>>::Error>(&#attribute_access_ident, &#qname)?;
        }
    });

    let deserialization_impl = match &data_struct.fields {
        syn::Fields::Unnamed(fields) if fields.unnamed.len() != 1 => simple_compile_error("Only tuple structs with 1 element are supported"),
        syn::Fields::Unnamed(_) => {
            quote! {
                ::core::str::FromStr::from_str(::xmlity::de::AttributeAccess::value(&#attribute_access_ident))
                    .map(#ident)
                    .map_err(::xmlity::de::Error::custom)
            }
        }
        syn::Fields::Named(_) =>
            simple_compile_error("Named fields in structs are not supported. Only tuple structs with 1 element are supported"),
        syn::Fields::Unit =>
            simple_compile_error("Unit structs are not supported. Only tuple structs with 1 element are supported"),
    };

    Ok(visit_attribute_fn_signature(
        &attribute_access_ident,
        visitor_lifetime,
        quote! {
            #xml_name_identification

            #deserialization_impl
        },
    ))
}

fn struct_derive_implementation(
    ident: &syn::Ident,
    generics: &syn::Generics,
    visitor_ident: &syn::Ident,
    visitor_lifetime: &syn::Lifetime,
    deserializer_ident: &syn::Ident,
    data_struct: &DataStruct,
    element_opts: Option<&crate::XmlityRootElementDeriveOpts>,
    attribute_opts: Option<&crate::XmlityRootAttributeDeriveOpts>,
    _value_opts: Option<&crate::XmlityRootValueDeriveOpts>,
) -> darling::Result<proc_macro2::TokenStream> {
    let formatter_expecting = format!("struct {}", ident);

    let visit_element_fn = element_opts.map(
        |crate::XmlityRootElementDeriveOpts {
             name,
             namespace,
             children_order,
             allow_unknown_attributes,
             attribute_order,
             allow_unknown_children,
             deserialize_any_name,
             ..
         }| {
            let ident_name = ident.to_string();
            let expanded_name = ExpandedName::new(
                name.0.as_ref().unwrap_or(&ident_name),
                namespace.0.as_deref(),
            );
            let expanded_name = if *deserialize_any_name {
                None
            } else {
                Some(expanded_name)
            };

            element_visit_fn(
                ident,
                visitor_lifetime,
                expanded_name,
                data_struct,
                *children_order,
                *allow_unknown_children,
                *attribute_order,
                *allow_unknown_attributes,
            )
        },
    );

    let visit_element_fn = match visit_element_fn {
        Some(element_visitor) => Some(element_visitor?),
        None => None,
    };

    let visit_attribute_fn = attribute_opts.map(
        |crate::XmlityRootAttributeDeriveOpts {
             name,
             namespace,
             deserialize_any_name,
             ..
         }| {
            let ident_name = ident.to_string();
            let expanded_name = ExpandedName::new(
                name.0.as_ref().unwrap_or(&ident_name),
                namespace.0.as_deref(),
            );
            let expanded_name = if *deserialize_any_name {
                None
            } else {
                Some(expanded_name)
            };

            attribute_visit_fn(ident, visitor_lifetime, expanded_name, data_struct)
        },
    );

    let visit_attribute_fn = match visit_attribute_fn {
        Some(attribute_visitor) => Some(attribute_visitor?),
        None => None,
    };

    let visitor_builder = VisitorBuilder::new(
        ident,
        generics,
        visitor_ident,
        visitor_lifetime,
        &formatter_expecting,
    )
    .visit_element_fn(visit_element_fn.unwrap_or_default())
    .visit_attribute_fn(visit_attribute_fn.unwrap_or_default());

    let visitor_def = visitor_builder.definition();
    let visitor_trait_impl = visitor_builder.trait_impl();

    Ok(quote! {
        #visitor_def

        #visitor_trait_impl

        ::xmlity::de::Deserializer::deserialize_any(#deserializer_ident, #visitor_ident {
            lifetime: ::core::marker::PhantomData,
            marker: ::core::marker::PhantomData,
        })
    })
}

fn visit_element_data_fn_impl_builder_field_decl(
    element_fields: impl IntoIterator<
        Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementDeriveOpts>,
    >,
    attribute_fields: impl IntoIterator<
        Item = DeserializeBuilderField<FieldIdent, XmlityFieldAttributeDeriveOpts>,
    >,
    group_fields: impl IntoIterator<
        Item = DeserializeBuilderField<FieldIdent, XmlityFieldGroupDeriveOpts>,
    >,
) -> proc_macro2::TokenStream {
    let getter_declarations = attribute_fields
        .into_iter()
        .map(
            |DeserializeBuilderField {
                 builder_field_ident,
                 field_type,
                 ..
             }| {
                quote! {
                    let mut #builder_field_ident = ::core::option::Option::<#field_type>::None;
                }
            },
        )
        .chain(element_fields.into_iter().map(
            |DeserializeBuilderField {
                 builder_field_ident,
                 field_type,
                 ..
             }| {
                quote! {
                    let mut #builder_field_ident = ::core::option::Option::<#field_type>::None;
                }
            },
        )).chain(group_fields.into_iter().map(
            |DeserializeBuilderField {
                 builder_field_ident,
                 field_type,
                 ..
             }| {
                quote! {
                    let mut #builder_field_ident = <#field_type as ::xmlity::de::DeserializationGroup>::builder();
                }
            },
        ));

    quote! {
        #(#getter_declarations)*
    }
}

fn visit_element_data_fn_impl_constructor(
    ident: &syn::Ident,
    visitor_lifetime: &syn::Lifetime,
    element_fields: impl IntoIterator<
        Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementDeriveOpts>,
    >,
    attribute_fields: impl IntoIterator<
        Item = DeserializeBuilderField<FieldIdent, XmlityFieldAttributeDeriveOpts>,
    >,
    group_fields: impl IntoIterator<
        Item = DeserializeBuilderField<FieldIdent, XmlityFieldGroupDeriveOpts>,
    >,
    constructor_type: StructType,
) -> proc_macro2::TokenStream {
    let local_value_expressions_constructors = attribute_fields.into_iter()
        .map(|a| a.map_options(XmlityFieldDeriveOpts::Attribute))
        .chain(element_fields.into_iter().map(|a| a.map_options(XmlityFieldDeriveOpts::Element)))
        .map(|DeserializeBuilderField { builder_field_ident, field_ident, options, .. }| {
            let expression = if matches!(options, XmlityFieldDeriveOpts::Element(XmlityFieldElementDeriveOpts {default: true}) | XmlityFieldDeriveOpts::Attribute(XmlityFieldAttributeDeriveOpts {default: true})) {
                quote! {
                    ::core::option::Option::unwrap_or_default(#builder_field_ident)
                }
            } else {
                quote! {
                    ::core::option::Option::ok_or(#builder_field_ident, ::xmlity::de::Error::missing_field(stringify!(#field_ident)))?
                }
            };
            (field_ident, expression)
        });
    let group_value_expressions_constructors = group_fields.into_iter().map(
        |DeserializeBuilderField {
             builder_field_ident,
             field_ident,
             ..
         }| {
            let expression = quote! {
                ::xmlity::de::DeserializationGroupBuilder::finish::<<A as ::xmlity::de::AttributesAccess<#visitor_lifetime>>::Error>(#builder_field_ident)?
            };

            (field_ident, expression)
        },
    );

    let value_expressions_constructors =
        local_value_expressions_constructors.chain(group_value_expressions_constructors);

    constructor_expr(ident, value_expressions_constructors, &constructor_type)
}

fn visit_element_data_fn_impl_attribute(
    access_ident: &Ident,
    span: proc_macro2::Span,
    fields: impl IntoIterator<
            Item = DeserializeBuilderField<FieldIdent, XmlityFieldAttributeGroupDeriveOpts>,
        > + Clone,
    allow_unknown_attributes: bool,
    order: ElementOrder,
) -> proc_macro2::TokenStream {
    let field_visits = builder_attribute_field_visitor(
        access_ident,
        quote! {},
        fields.clone(),
        quote! {break;},
        match order {
            ElementOrder::Loose => quote! {break;},
            ElementOrder::None => quote! {continue;},
        },
        quote! {continue;},
        quote! {},
        match order {
            ElementOrder::Loose => true,
            ElementOrder::None => false,
        },
    );

    match order {
        ElementOrder::Loose => field_visits.zip(fields).map(|(field_visit, field)| {
            let skip_unknown = if allow_unknown_attributes {
                let skip_ident = Ident::new("__skip", span);
                quote! {
                    let #skip_ident = ::xmlity::de::AttributesAccess::next_attribute::<::xmlity::types::utils::IgnoredAny>(&mut #access_ident).unwrap_or(None);
                    if ::core::option::Option::is_none(&#skip_ident) {
                        break;
                    }
                    continue;
                }
            } else {
                let condition = attribute_done(field, quote! {});

                quote! {
                    if #condition {
                        break;
                    } else {
                        return ::core::result::Result::Err(::xmlity::de::Error::unknown_child());
                    }
                }
            };

            quote! {
                loop {
                    #field_visit
                    #skip_unknown
                }
            }
        }).collect(),
        ElementOrder::None => {
            let skip_unknown = if allow_unknown_attributes {
                let skip_ident = Ident::new("__skip", span);
                quote! {
                    let #skip_ident = ::xmlity::de::AttributesAccess::next_attribute::<::xmlity::types::utils::IgnoredAny>(&mut #access_ident).unwrap_or(None);
                    if ::core::option::Option::is_none(&#skip_ident) {
                        break
                    }
                }
            } else {
                let all_some_condition = all_attributes_done(fields, quote! {});

                quote! {
                    if #all_some_condition {
                        break;
                    } else {
                        return ::core::result::Result::Err(::xmlity::de::Error::unknown_child());
                    }
                }
            };

            quote! {
                loop {
                    #(#field_visits)*
                    #skip_unknown
                }
            }
        },
    }
}

fn visit_element_data_fn_impl_children(
    element_access_ident: &syn::Ident,
    fields: impl IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementGroupDeriveOpts>>
        + Clone,
    allow_unknown_children: bool,
    order: ElementOrder,
) -> proc_macro2::TokenStream {
    let access_ident = Ident::new("__children", element_access_ident.span());

    let field_visits = builder_element_field_visitor(
        &access_ident,
        quote! {},
        fields.clone(),
        quote! {break;},
        match order {
            ElementOrder::Loose => quote! {break;},
            ElementOrder::None => quote! {continue;},
        },
        quote! {continue;},
        quote! {},
        match order {
            ElementOrder::Loose => true,
            ElementOrder::None => false,
        },
    );

    let access_loop = match order {
        ElementOrder::Loose => field_visits.zip(fields).map(|(field_visit, field)| {
            let skip_unknown = if allow_unknown_children {
                let skip_ident = Ident::new("__skip", element_access_ident.span());
                quote! {
                    let #skip_ident = ::xmlity::de::SeqAccess::next_element::<::xmlity::types::utils::IgnoredAny>(&mut #access_ident).unwrap_or(None);
                    if ::core::option::Option::is_none(&#skip_ident) {
                        break;
                    }
                    continue;
                }
            } else {
                let condition = element_done(field, quote! {});

                quote! {
                    if #condition {
                        break;
                    } else {
                        return ::core::result::Result::Err(::xmlity::de::Error::unknown_child());
                    }
                }
            };


            quote! {
                loop {
                    #field_visit
                    #skip_unknown
                }
            }
        }).collect(),
        ElementOrder::None => {
            let skip_unknown = if allow_unknown_children {
                let skip_ident = Ident::new("__skip", element_access_ident.span());
                quote! {
                    let #skip_ident = ::xmlity::de::SeqAccess::next_element::<::xmlity::types::utils::IgnoredAny>(&mut #access_ident).unwrap_or(None);
                    if ::core::option::Option::is_none(&#skip_ident) {
                        break
                    }
                }
            } else {
                let all_some_condition = all_elements_done(fields, quote! {});

                quote! {
                    if #all_some_condition {
                        break;
                    } else {
                        return ::core::result::Result::Err(::xmlity::de::Error::unknown_child());
                    }
                }
            };

            quote! {
                loop {
                    #(#field_visits)*
                    #skip_unknown
                }
            }
        },
    };

    quote! {
        let mut #access_ident = ::xmlity::de::ElementAccess::children(#element_access_ident)?;

        #access_loop
    }
}

fn struct_fields_visitor_end<T>(
    struct_ident: &syn::Ident,
    visitor_lifetime: &syn::Lifetime,
    fields: T,
    constructor_type: StructType,
    children_order: ElementOrder,
    allow_unknown_children: bool,
    attributes_order: ElementOrder,
    allow_unknown_attributes: bool,
) -> proc_macro2::TokenStream
where
    T: IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldDeriveOpts>> + Clone,
    T::IntoIter: Clone,
{
    let element_access_ident = syn::Ident::new("__element", struct_ident.span());

    let element_fields = fields.clone().into_iter().filter_map(|field| {
        field.map_options_opt(|opt| match opt {
            XmlityFieldDeriveOpts::Element(opts) => Some(opts),
            _ => None,
        })
    });

    let attribute_fields = fields.clone().into_iter().filter_map(|field| {
        field.map_options_opt(|opt| match opt {
            XmlityFieldDeriveOpts::Attribute(opts) => Some(opts),
            _ => None,
        })
    });

    let group_fields = fields.clone().into_iter().filter_map(|field| {
        field.map_options_opt(|opt| match opt {
            XmlityFieldDeriveOpts::Group(opts) => Some(opts),
            _ => None,
        })
    });

    let getter_declarations = visit_element_data_fn_impl_builder_field_decl(
        element_fields.clone(),
        attribute_fields.clone(),
        group_fields.clone(),
    );

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

    let attribute_loop = if attribute_group_fields.clone().next().is_some() {
        Some(visit_element_data_fn_impl_attribute(
            &element_access_ident,
            struct_ident.span(),
            attribute_group_fields,
            allow_unknown_attributes,
            attributes_order,
        ))
    } else {
        None
    };

    let children_loop = if element_group_fields.clone().next().is_some() {
        Some(visit_element_data_fn_impl_children(
            &element_access_ident,
            element_group_fields,
            allow_unknown_children,
            children_order,
        ))
    } else {
        None
    };

    let constructor = visit_element_data_fn_impl_constructor(
        struct_ident,
        visitor_lifetime,
        element_fields.clone(),
        attribute_fields.clone(),
        group_fields.clone(),
        constructor_type,
    );

    quote! {
        #getter_declarations

        #attribute_loop

        #children_loop

        ::core::result::Result::Ok(#constructor)
    }
}

fn visit_element_unit_fn_impl(ident: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        ::core::result::Result::Ok(#ident)
    }
}

fn visit_seq_fn_signature(
    seq_acces_ident: &syn::Ident,
    visitor_lifetime: &syn::Lifetime,
    body: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        fn visit_seq<S>(self, mut #seq_acces_ident: S) -> Result<Self::Value, <S as xmlity::de::SeqAccess<#visitor_lifetime>>::Error>
        where
            S: xmlity::de::SeqAccess<#visitor_lifetime>,
        {
            #body
        }
    }
}

fn visit_text_fn_signature(
    value_ident: &syn::Ident,
    body: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        fn visit_text<E, V>(self, #value_ident: V) -> Result<Self::Value, E>
        where
            E: ::xmlity::de::Error,
            V: ::xmlity::de::XmlText,
        {
            #body
        }
    }
}

fn visit_cdata_fn_signature(
    value_ident: &syn::Ident,
    body: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        fn visit_cdata<E, V>(self, #value_ident: V) -> Result<Self::Value, E>
        where
            E: ::xmlity::de::Error,
            V: ::xmlity::de::XmlCData,
        {
            #body
        }
    }
}

fn enum_derive_implementation<'a>(
    ident: &syn::Ident,
    generics: &syn::Generics,
    visitor_ident: &syn::Ident,
    visitor_lifetime: &syn::Lifetime,
    deserializer_ident: &syn::Ident,
    variants: impl IntoIterator<Item = &'a syn::Variant> + Clone,
    _element_opts: Option<&crate::XmlityRootElementDeriveOpts>,
    _attribute_opts: Option<&crate::XmlityRootAttributeDeriveOpts>,
    value_opts: Option<&crate::XmlityRootValueDeriveOpts>,
) -> proc_macro2::TokenStream {
    let visit_seq_fn = (|| {
        let sequence_access_ident = syn::Ident::new("__sequence", Span::mixed_site());

        let variants = variants.clone().into_iter().map(|Variant {
            ident: variant_ident,
            fields: variant_fields,
            ..
        }| {
            match variant_fields {
                syn::Fields::Named(_fields) => {
                    None
                }
                syn::Fields::Unnamed(fields) if fields.unnamed.len() != 1 => {
                    None
                }
                syn::Fields::Unnamed(fields) => {
                    let Field {
                        ty: field_type,
                        ..
                    } = fields.unnamed.first().expect("This is guaranteed by the check above");

                    Some(quote! {
                        if let ::core::result::Result::Ok(::core::option::Option::Some(_v)) = ::xmlity::de::SeqAccess::next_element::<#field_type>(&mut #sequence_access_ident) {
                            return ::core::result::Result::Ok(#ident::#variant_ident(_v));
                        }
                    })
                }
                syn::Fields::Unit =>{
                    None
                },
            }
        }).collect::<Option<Vec<_>>>()?;
        let ident_string = ident.to_string();

        Some(visit_seq_fn_signature(
            &sequence_access_ident,
            visitor_lifetime,
            quote! {
                #(#variants)*

                ::core::result::Result::Err(::xmlity::de::Error::no_possible_variant(#ident_string))
            },
        ))
    })();

    let (visit_text_fn, visit_cdata_fn) = value_opts.map(|value_opts| (|| {
        let value_ident_owned = Ident::new("__valueowned", Span::mixed_site());
        let value_ident = Ident::new("__value", Span::mixed_site());
        let variants = variants.clone().into_iter().map(|Variant {
            ident: variant_ident,
            fields: variant_fields,
            ..
        }| {
            let variant_ident_string = value_opts.rename_all.apply_to_variant(&variant_ident.to_string());
            match variant_fields {
                syn::Fields::Named(_fields) => {
                    None
                }
                syn::Fields::Unnamed(_fields) => {
                    None
                }
                syn::Fields::Unit =>{
                    Some(quote! {
                        if ::core::primitive::str::trim(::core::ops::Deref::deref(&#value_ident)) == #variant_ident_string {
                            return ::core::result::Result::Ok(#ident::#variant_ident);
                        }
                    })
                },
            }
        }).collect::<Option<Vec<_>>>();

        let variants = match variants {
            Some(variants) => variants,
            None => return (None, None),
        };

        let ident_string = ident.to_string();

        let fn_body = quote! {
            #(#variants)*

            ::core::result::Result::Err(::xmlity::de::Error::no_possible_variant(#ident_string))
        };

        (Some(visit_text_fn_signature(&value_ident_owned, quote! {
            let #value_ident = ::xmlity::de::XmlText::as_str(&#value_ident_owned);
            #fn_body
        })), Some(visit_cdata_fn_signature(&value_ident_owned, quote! {
            let #value_ident = ::xmlity::de::XmlCData::as_str(&#value_ident_owned);
            #fn_body
        })))
    })()).unwrap_or_default();

    let formatter_expecting = format!("enum {}", ident);

    let visitor_builder = VisitorBuilder::new(
        ident,
        generics,
        visitor_ident,
        visitor_lifetime,
        &formatter_expecting,
    )
    .visit_seq_fn(visit_seq_fn)
    .visit_text_fn(visit_text_fn)
    .visit_cdata_fn(visit_cdata_fn);

    let visitor_def = visitor_builder.definition();
    let visitor_trait_impl = visitor_builder.trait_impl();

    let deserialize_expr = if value_opts.is_some() {
        quote! {
            ::xmlity::de::Deserializer::deserialize_any(#deserializer_ident, #visitor_ident {
                lifetime: ::core::marker::PhantomData,
                marker: ::core::marker::PhantomData,
            })
        }
    } else {
        quote! {
            ::xmlity::de::Deserializer::deserialize_seq(#deserializer_ident, #visitor_ident {
                lifetime: ::core::marker::PhantomData,
                marker: ::core::marker::PhantomData,
            })
        }
    };

    quote! {
        #visitor_def
        #visitor_trait_impl
        #deserialize_expr
    }
}

pub fn derive_deserialize_fn(
    ast: syn::DeriveInput,
    element_opts: Option<crate::XmlityRootElementDeriveOpts>,
    attribute_opts: Option<crate::XmlityRootAttributeDeriveOpts>,
    value_opts: Option<crate::XmlityRootValueDeriveOpts>,
) -> darling::Result<proc_macro2::TokenStream> {
    let visitor_ident = Ident::new(format!("__{}Visitor", ast.ident).as_str(), ast.ident.span());
    let visitor_lifetime = Lifetime::new("'__visitor", ast.ident.span());
    let deserializer_ident = Ident::new("__deserializer", ast.ident.span());
    let deserialize_lifetime = Lifetime::new("'__deserialize", ast.ident.span());
    let implementation = match &ast.data {
        syn::Data::Struct(data_struct) => struct_derive_implementation(
            &ast.ident,
            &ast.generics,
            &visitor_ident,
            &visitor_lifetime,
            &deserializer_ident,
            data_struct,
            element_opts.as_ref(),
            attribute_opts.as_ref(),
            value_opts.as_ref(),
        )?,
        syn::Data::Enum(DataEnum { variants, .. }) => enum_derive_implementation(
            &ast.ident,
            &ast.generics,
            &visitor_ident,
            &visitor_lifetime,
            &deserializer_ident,
            variants.iter(),
            element_opts.as_ref(),
            attribute_opts.as_ref(),
            value_opts.as_ref(),
        ),
        syn::Data::Union(_) => simple_compile_error("Unions are not supported yet"),
    };

    let trait_impl_builder = DeserializeTraitImplBuilder::new(
        &ast.ident,
        &ast.generics,
        &deserializer_ident,
        &deserialize_lifetime,
        implementation,
    );

    Ok(trait_impl_builder.trait_impl())
}
