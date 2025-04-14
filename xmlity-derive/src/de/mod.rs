mod element;
mod group;
pub use element::DeriveDeserialize;
pub use group::DeriveDeserializationGroup;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_quote, Ident, Stmt, Type, Visibility};

use crate::{
    options::{
        XmlityFieldAttributeDeriveOpts, XmlityFieldElementDeriveOpts, XmlityFieldGroupDeriveOpts,
    },
    utils::{self},
    DeserializeBuilderField, FieldIdent, XmlityFieldAttributeGroupDeriveOpts,
    XmlityFieldElementGroupDeriveOpts,
};

mod common;

#[derive(Clone, Copy)]
enum StructType {
    Named,
    Unnamed,
}

fn named_constructor_expr<I: ToTokens, K: ToTokens, V: ToTokens>(
    ident: I,
    fields: impl IntoIterator<Item = (K, V)>,
) -> proc_macro2::TokenStream {
    let field_tokens = fields.into_iter().map(|(ident, expression)| {
        quote! {
            #ident: #expression,
        }
    });

    quote! {
        #ident {
            #(#field_tokens)*
        }
    }
}

fn unnamed_constructor_expr<I: ToTokens, T: ToTokens>(
    ident: I,
    fields: impl IntoIterator<Item = T>,
) -> proc_macro2::TokenStream {
    let fields = fields.into_iter();

    quote! {
      #ident (
        #(#fields,)*
    )
    }
}

fn constructor_expr<I: ToTokens, T: ToTokens>(
    ident: I,
    fields: impl IntoIterator<Item = (FieldIdent, T)>,
    constructor_type: &StructType,
) -> proc_macro2::TokenStream {
    let fields = fields.into_iter();
    match constructor_type {
        StructType::Unnamed => {
            unnamed_constructor_expr(ident, fields.map(|(_, value_expression)| value_expression))
        }
        StructType::Named => named_constructor_expr(
            ident,
            fields.filter_map(|(a, value_expression)| match a {
                FieldIdent::Named(field_ident) => Some((field_ident, value_expression)),
                FieldIdent::Indexed(_) => None,
            }),
        ),
    }
}

fn named_struct_definition_expr<I: ToTokens, K: ToTokens, V: ToTokens>(
    ident: I,
    generics: Option<&syn::Generics>,
    fields: impl IntoIterator<Item = (K, V)>,
    visibility: &Visibility,
) -> proc_macro2::TokenStream {
    let field_tokens = fields.into_iter().map(|(ident, expression)| {
        quote! {
            #ident: #expression,
        }
    });

    quote! {
        #visibility struct #ident #generics {
            #(#field_tokens)*
        }
    }
}

fn unnamed_struct_definition_expr<I: ToTokens, T: ToTokens>(
    ident: I,
    generics: Option<&syn::Generics>,
    fields: impl IntoIterator<Item = T>,
    visibility: &Visibility,
) -> proc_macro2::TokenStream {
    let fields = fields.into_iter();

    quote! {
        #visibility struct #ident #generics (
            #(#fields,)*
        )
    }
}

fn struct_definition_expr<I: ToTokens>(
    ident: I,
    generics: Option<&syn::Generics>,
    fields: impl IntoIterator<Item = (FieldIdent, Type)>,
    constructor_type: &StructType,
    visibility: &Visibility,
) -> proc_macro2::TokenStream {
    let fields = fields.into_iter();
    match constructor_type {
        StructType::Unnamed => unnamed_struct_definition_expr(
            ident,
            generics,
            fields.map(|(_, value_expression)| value_expression),
            visibility,
        ),
        StructType::Named => named_struct_definition_expr(
            ident,
            generics,
            fields.filter_map(|(a, value_expression)| match a {
                FieldIdent::Named(field_ident) => Some((field_ident, value_expression)),
                FieldIdent::Indexed(_) => None,
            }),
            visibility,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn builder_attribute_field_visitor<
    F: IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldAttributeGroupDeriveOpts>>,
>(
    access_ident: &Ident,
    builder_field_ident_prefix: proc_macro2::TokenStream,
    fields: F,
    if_next_attribute_none: Vec<Stmt>,
    finished_attribute: Vec<Stmt>,
    if_contributed_to_groups: Vec<Stmt>,
    after_attempt: Vec<Stmt>,
    pop_error: bool,
) -> Vec<Stmt> {
    fn attribute_field_deserialize_impl(
        access_ident: &Ident,
        builder_field_ident_prefix: impl ToTokens,
        DeserializeBuilderField {
            builder_field_ident,
            field_type,
            ..
        }: DeserializeBuilderField<FieldIdent, XmlityFieldAttributeDeriveOpts>,
        if_next_attribute_none: &[Stmt],
        finished_attribute: &[Stmt],
        after_attempt: &[Stmt],
        pop_error: bool,
    ) -> Vec<Stmt> {
        let temporary_value_ident = Ident::new("__v", Span::call_site());

        if pop_error {
            parse_quote! {
                if ::core::option::Option::is_none(&#builder_field_ident_prefix #builder_field_ident) {
                    let #temporary_value_ident = ::xmlity::de::AttributesAccess::next_attribute::<#field_type>(&mut #access_ident)?;
                    let ::core::option::Option::Some(#temporary_value_ident) = #temporary_value_ident else {
                        #(#if_next_attribute_none)*
                    };
                    #builder_field_ident_prefix #builder_field_ident = ::core::option::Option::Some(#temporary_value_ident);
                    #(#finished_attribute)*

                }
            }
        } else {
            parse_quote! {
                if ::core::option::Option::is_none(&#builder_field_ident_prefix #builder_field_ident) {
                    if let ::core::result::Result::Ok(#temporary_value_ident) = ::xmlity::de::AttributesAccess::next_attribute::<#field_type>(&mut #access_ident) {
                        let ::core::option::Option::Some(#temporary_value_ident) = #temporary_value_ident else {
                            #(#if_next_attribute_none)*
                        };
                        #builder_field_ident_prefix #builder_field_ident = ::core::option::Option::Some(#temporary_value_ident);
                        #(#finished_attribute)*
                    }
                    #(#after_attempt)*
                }
            }
        }
    }

    fn group_field_deserialize_impl(
        access_ident: &Ident,
        builder_field_ident_prefix: impl ToTokens,
        DeserializeBuilderField {
            builder_field_ident,
            ..
        }: DeserializeBuilderField<FieldIdent, XmlityFieldGroupDeriveOpts>,
        if_contributed_to_groups: &[Stmt],
        after_attempt: &[Stmt],
        pop_error: bool,
    ) -> Vec<Stmt> {
        let contributed_to_attributes_ident =
            Ident::new("__contributed_to_attributes", Span::call_site());

        if pop_error {
            parse_quote! {
                if !::xmlity::de::DeserializationGroupBuilder::attributes_done(&#builder_field_ident_prefix #builder_field_ident) {
                    let #contributed_to_attributes_ident = ::xmlity::de::DeserializationGroupBuilder::contribute_attributes(&mut #builder_field_ident_prefix #builder_field_ident, ::xmlity::de::AttributesAccess::sub_access(&mut #access_ident)?)?;
                    if #contributed_to_attributes_ident {
                        #(#if_contributed_to_groups)*
                    }
                }
            }
        } else {
            parse_quote! {
                if !::xmlity::de::DeserializationGroupBuilder::attributes_done(&#builder_field_ident_prefix #builder_field_ident) {
                    if let ::core::result::Result::Ok(#contributed_to_attributes_ident) = ::xmlity::de::DeserializationGroupBuilder::contribute_attributes(&mut #builder_field_ident_prefix #builder_field_ident, ::xmlity::de::AttributesAccess::sub_access(&mut #access_ident)?) {
                        if #contributed_to_attributes_ident {
                            #(#if_contributed_to_groups)*
                        }
                    }
                    #(#after_attempt)*
                }
            }
        }
    }

    fields
        .into_iter()
        .zip(utils::repeat_clone((
            builder_field_ident_prefix,
            if_next_attribute_none,
            finished_attribute,
            if_contributed_to_groups,
            after_attempt,
        )))
        .map(
            move |(
                var_field,
                (
                    builder_field_ident_prefix,
                    if_next_attribute_none,
                    finished_attribute,
                    if_contributed_to_groups,
                    after_attempt,
                ),
            )| match &var_field.options {
                XmlityFieldAttributeGroupDeriveOpts::Attribute(_) => {
                    attribute_field_deserialize_impl(
                        access_ident,
                        builder_field_ident_prefix,
                        var_field.map_options(|opts| match opts {
                            XmlityFieldAttributeGroupDeriveOpts::Attribute(opts) => opts,
                            _ => unreachable!(),
                        }),
                        if_next_attribute_none.as_slice(),
                        finished_attribute.as_slice(),
                        after_attempt.as_slice(),
                        pop_error,
                    )
                }
                XmlityFieldAttributeGroupDeriveOpts::Group(_) => group_field_deserialize_impl(
                    access_ident,
                    builder_field_ident_prefix,
                    var_field.map_options(|opts| match opts {
                        XmlityFieldAttributeGroupDeriveOpts::Group(opts) => opts,
                        _ => unreachable!(),
                    }),
                    if_contributed_to_groups.as_slice(),
                    after_attempt.as_slice(),
                    pop_error,
                ),
            },
        )
        .flatten()
        .collect::<Vec<_>>()
}

#[allow(clippy::too_many_arguments)]
fn builder_element_field_visitor<
    F: IntoIterator<Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementGroupDeriveOpts>>,
>(
    access_ident: &Ident,
    builder_field_ident_prefix: proc_macro2::TokenStream,
    fields: F,
    if_next_element_none: Vec<Stmt>,
    finished_element: Vec<Stmt>,
    if_contributed_to_groups: Vec<Stmt>,
    after_attempt: Vec<Stmt>,
    pop_error: bool,
) -> Vec<Stmt> {
    fn element_field_deserialize_impl(
        access_ident: &Ident,
        builder_field_ident_prefix: impl ToTokens,
        DeserializeBuilderField {
            builder_field_ident,
            field_type,
            ..
        }: DeserializeBuilderField<FieldIdent, XmlityFieldElementDeriveOpts>,
        if_next_element_none: &[Stmt],
        finished_element: &[Stmt],
        after_attempt: &[Stmt],
        pop_error: bool,
    ) -> Vec<Stmt> {
        let temporary_value_ident = Ident::new("__v", Span::call_site());

        if pop_error {
            parse_quote! {
                if ::core::option::Option::is_none(&#builder_field_ident_prefix #builder_field_ident) {
                    let #temporary_value_ident = ::xmlity::de::SeqAccess::next_element_seq::<#field_type>(&mut #access_ident)?;
                    let ::core::option::Option::Some(#temporary_value_ident) = #temporary_value_ident else {
                        #(#if_next_element_none)*
                    };
                    #builder_field_ident_prefix #builder_field_ident = ::core::option::Option::Some(#temporary_value_ident);

                    #(#finished_element)*
                }
            }
        } else {
            parse_quote! {
                if ::core::option::Option::is_none(&#builder_field_ident_prefix #builder_field_ident) {
                    if let ::core::result::Result::Ok(#temporary_value_ident) = ::xmlity::de::SeqAccess::next_element_seq::<#field_type>(&mut #access_ident) {
                        let ::core::option::Option::Some(#temporary_value_ident) = #temporary_value_ident else {
                            #(#if_next_element_none)*
                        };
                        #builder_field_ident_prefix #builder_field_ident = ::core::option::Option::Some(#temporary_value_ident);

                        #(#finished_element)*
                    }
                    #(#after_attempt)*
                }
            }
        }
    }

    fn group_field_deserialize_impl(
        access_ident: &Ident,
        builder_field_ident_prefix: impl ToTokens,
        DeserializeBuilderField {
            builder_field_ident,
            ..
        }: DeserializeBuilderField<FieldIdent, XmlityFieldGroupDeriveOpts>,
        if_contributed_to_groups: &[Stmt],
        after_attempt: &[Stmt],
        pop_error: bool,
    ) -> Vec<Stmt> {
        let contributed_to_elements_ident =
            Ident::new("__contributed_to_elements", Span::call_site());

        if pop_error {
            parse_quote! {
                if !::xmlity::de::DeserializationGroupBuilder::elements_done(&#builder_field_ident_prefix #builder_field_ident) {
                    let #contributed_to_elements_ident = ::xmlity::de::DeserializationGroupBuilder::contribute_elements(&mut #builder_field_ident_prefix #builder_field_ident, ::xmlity::de::SeqAccess::sub_access(&mut #access_ident)?)?;
                    if #contributed_to_elements_ident {
                        #(#if_contributed_to_groups)*
                    }

                }
            }
        } else {
            parse_quote! {
                if !::xmlity::de::DeserializationGroupBuilder::elements_done(&#builder_field_ident_prefix #builder_field_ident) {
                    if let ::core::result::Result::Ok(#contributed_to_elements_ident) = ::xmlity::de::DeserializationGroupBuilder::contribute_elements(&mut #builder_field_ident_prefix #builder_field_ident, ::xmlity::de::SeqAccess::sub_access(&mut #access_ident)?) {
                        if #contributed_to_elements_ident {
                            #(#if_contributed_to_groups)*
                        }
                    }
                    #(#after_attempt)*
                }
            }
        }
    }

    fields
        .into_iter()
        .zip(utils::repeat_clone((
            builder_field_ident_prefix,
            if_next_element_none,
            finished_element,
            if_contributed_to_groups,
            after_attempt,
        )))
        .map(
            move |(
                var_field,
                (
                    builder_field_ident_prefix,
                    if_next_element_none,
                    finished_element,
                    if_contributed_to_groups,
                    after_attempt,
                ),
            )| match &var_field.options {
                XmlityFieldElementGroupDeriveOpts::Element(_) => element_field_deserialize_impl(
                    access_ident,
                    builder_field_ident_prefix,
                    var_field.map_options(|opts| match opts {
                        XmlityFieldElementGroupDeriveOpts::Element(opts) => opts,
                        _ => unreachable!(),
                    }),
                    if_next_element_none.as_slice(),
                    finished_element.as_slice(),
                    after_attempt.as_slice(),
                    pop_error,
                ),
                XmlityFieldElementGroupDeriveOpts::Group(_) => group_field_deserialize_impl(
                    access_ident,
                    builder_field_ident_prefix,
                    var_field.map_options(|opts| match opts {
                        XmlityFieldElementGroupDeriveOpts::Group(opts) => opts,
                        _ => unreachable!(),
                    }),
                    if_contributed_to_groups.as_slice(),
                    after_attempt.as_slice(),
                    pop_error,
                ),
            },
        )
        .flatten()
        .collect::<Vec<_>>()
}

fn attribute_done(
    field: DeserializeBuilderField<FieldIdent, XmlityFieldAttributeGroupDeriveOpts>,
    builder_field_ident_prefix: impl ToTokens,
) -> proc_macro2::TokenStream {
    let DeserializeBuilderField {
        builder_field_ident,
        options,
        ..
    } = field;
    match options {
        XmlityFieldAttributeGroupDeriveOpts::Attribute(_) => quote! {
            ::core::option::Option::is_some(&#builder_field_ident_prefix #builder_field_ident)
        },
        XmlityFieldAttributeGroupDeriveOpts::Group(_) => quote! {
            ::xmlity::de::DeserializationGroupBuilder::attributes_done(&#builder_field_ident_prefix #builder_field_ident)
        },
    }
}

fn all_attributes_done(
    fields: impl IntoIterator<
        Item = DeserializeBuilderField<FieldIdent, XmlityFieldAttributeGroupDeriveOpts>,
    >,

    builder_field_ident_prefix: impl ToTokens,
) -> proc_macro2::TokenStream {
    let conditions = fields
        .into_iter()
        .map(|field| attribute_done(field, &builder_field_ident_prefix));

    let conditions = quote! {
        #(#conditions)&&*
    };

    if conditions.is_empty() {
        quote! {true}
    } else {
        quote! {#conditions}
    }
}

fn element_done(
    field: DeserializeBuilderField<FieldIdent, XmlityFieldElementGroupDeriveOpts>,
    builder_field_ident_prefix: impl ToTokens,
) -> proc_macro2::TokenStream {
    let DeserializeBuilderField {
        builder_field_ident,
        options,
        ..
    } = field;
    match options {
        XmlityFieldElementGroupDeriveOpts::Element(_) => quote! {
            ::core::option::Option::is_some(&#builder_field_ident_prefix #builder_field_ident)
        },
        XmlityFieldElementGroupDeriveOpts::Group(_) => quote! {
            ::xmlity::de::DeserializationGroupBuilder::elements_done(&#builder_field_ident_prefix #builder_field_ident)
        },
    }
}

fn all_elements_done(
    fields: impl IntoIterator<
        Item = DeserializeBuilderField<FieldIdent, XmlityFieldElementGroupDeriveOpts>,
    >,
    builder_field_ident_prefix: impl ToTokens,
) -> proc_macro2::TokenStream {
    let conditions = fields
        .into_iter()
        .map(|field| element_done(field, &builder_field_ident_prefix));

    let conditions = quote! {
        #(#conditions)&&*
    };

    if conditions.is_empty() {
        quote! {true}
    } else {
        quote! {#conditions}
    }
}
