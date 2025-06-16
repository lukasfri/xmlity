use proc_macro2::Span;
use syn::{parse_quote, Expr, ExprWhile, Generics, Ident, Stmt};

use crate::{
    common::FieldIdent,
    de::builders::DeserializeBuilderExt,
    options::{records::fields::FieldValueGroupOpts, FieldWithOpts},
    DeriveError, DeriveResult,
};

use crate::{
    options::{
        records::fields::{
            AttributeDeclaredOpts, AttributeOpts, ChildOpts, ElementOpts, FieldAttributeGroupOpts,
            FieldOpts, GroupOpts, ValueOpts,
        },
        Extendable, WithExpandedNameExt,
    },
    utils::{self},
};

use quote::{quote, ToTokens};

use super::deserialize::SimpleDeserializeAttributeBuilder;

fn attribute_field_deserialize_impl(
    access_expr: &Expr,
    ident_to_expr: impl FnOnce(&FieldIdent) -> Expr,
    FieldWithOpts {
        field_ident,
        field_type,
        options,
    }: FieldWithOpts<FieldIdent, AttributeOpts>,
    if_next_attribute_none: &[Stmt],
    finished_attribute: &[Stmt],
    after_attempt: &[Stmt],
    pop_error: bool,
) -> DeriveResult<Vec<Stmt>> {
    let builder_expr = ident_to_expr(&field_ident);
    let temporary_value_ident = Ident::new("__v", Span::call_site());
    let wrapper_ident = Ident::new("__W", Span::call_site());
    let empty_generics: Generics = parse_quote!();

    let wrapper_data = match &options {
        AttributeOpts::Declared(opts @ AttributeDeclaredOpts { .. }) => {
            let builder = SimpleDeserializeAttributeBuilder {
                ident: &wrapper_ident,
                generics: &empty_generics,
                required_expanded_name: Some(
                    opts.expanded_name(field_ident.to_named_ident().to_string().as_str())
                        .into_owned(),
                ),
                item_type: &field_type,
            };

            let deserialize_unwrapper = |ident: &Ident| {
                quote! {
                    let mut #ident = #ident.__value;
                }
            };

            Some((builder, deserialize_unwrapper))
        }
        _ => None,
    };

    let deserialize_type = wrapper_data
        .as_ref()
        .map(|(a, _)| a.ident)
        .map(|a| parse_quote!(#a))
        .unwrap_or(field_type.clone());

    let deserialize_wrapper_def: Vec<Stmt> = match wrapper_data.as_ref() {
        Some((a, _)) => {
            let def = a.struct_definition();
            let trait_impl = a.to_builder().deserialize_trait_impl()?;
            parse_quote!(
                #def
                #trait_impl
            )
        }
        None => Vec::new(),
    };

    let deserialize_unwrapper = wrapper_data.as_ref().map(|(_, a)| a);

    let value_transformer = deserialize_unwrapper.map(|a| (a)(&temporary_value_ident));

    let inner = quote! {
        let ::core::option::Option::Some(#temporary_value_ident) = #temporary_value_ident else {
            #(#if_next_attribute_none)*
        };
        #value_transformer
        #builder_expr = ::core::option::Option::Some(#temporary_value_ident);
        #(#finished_attribute)*
    };
    let deserialize_expr: Expr = parse_quote!(
        ::xmlity::de::AttributesAccess::next_attribute::<#deserialize_type>(#access_expr)
    );

    let inner = pop_or_ignore_error(&temporary_value_ident, &deserialize_expr, pop_error, inner);

    let after_attempt = if !pop_error { after_attempt } else { &[] };

    Ok(parse_quote! {
        if ::core::option::Option::is_none(&#builder_expr) {
            #(#deserialize_wrapper_def)*

            #inner

            #(#after_attempt)*
        }
    })
}

fn group_field_contribute_attributes(
    access_expr: &Expr,
    ident_to_expr: impl FnOnce(&FieldIdent) -> Expr,
    FieldWithOpts { field_ident, .. }: FieldWithOpts<FieldIdent, GroupOpts>,
    if_contributed_to_groups: &[Stmt],
    after_attempt: &[Stmt],
    pop_error: bool,
) -> Vec<Stmt> {
    let builder_expr = ident_to_expr(&field_ident);
    let contributed_to_attributes_ident =
        Ident::new("__contributed_to_attributes", Span::call_site());

    let deserialize_expr: Expr = parse_quote!(
        ::xmlity::de::DeserializationGroupBuilder::contribute_attributes(&mut #builder_expr, ::xmlity::de::AttributesAccess::sub_access(#access_expr)?)
    );

    let inner = quote! {
        if #contributed_to_attributes_ident {
            #(#if_contributed_to_groups)*
        }
    };

    let inner = pop_or_ignore_error(
        &contributed_to_attributes_ident,
        &deserialize_expr,
        pop_error,
        inner,
    );

    let after_attempt = if !pop_error { after_attempt } else { &[] };

    parse_quote! {
        if !::xmlity::de::DeserializationGroupBuilder::attributes_done(&#builder_expr) {
            #inner
            #(#after_attempt)*
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn builder_attribute_field_visitor<
    F: IntoIterator<Item = FieldWithOpts<FieldIdent, FieldAttributeGroupOpts>>,
>(
    access_expr: &Expr,
    ident_to_expr: impl Fn(&FieldIdent) -> Expr + Clone,
    fields: F,
    if_next_attribute_none: Vec<Stmt>,
    finished_attribute: Vec<Stmt>,
    if_contributed_to_groups: Vec<Stmt>,
    after_attempt: Vec<Stmt>,
    pop_error: bool,
) -> DeriveResult<Vec<Stmt>> {
    fields
        .into_iter()
        .zip(utils::repeat_clone((
            ident_to_expr,
            if_next_attribute_none,
            finished_attribute,
            if_contributed_to_groups,
            after_attempt,
        )))
        .map(
            move |(
                var_field,
                (
                    ident_to_expr,
                    if_next_attribute_none,
                    finished_attribute,
                    if_contributed_to_groups,
                    after_attempt,
                ),
            )| match &var_field.options {
                FieldAttributeGroupOpts::Attribute(_) => attribute_field_deserialize_impl(
                    access_expr,
                    ident_to_expr,
                    var_field.map_options(|opts| match opts {
                        FieldAttributeGroupOpts::Attribute(opts) => opts,
                        _ => unreachable!(),
                    }),
                    if_next_attribute_none.as_slice(),
                    finished_attribute.as_slice(),
                    after_attempt.as_slice(),
                    pop_error,
                ),
                FieldAttributeGroupOpts::Group(_) => Ok(group_field_contribute_attributes(
                    access_expr,
                    ident_to_expr,
                    var_field.map_options(|opts| match opts {
                        FieldAttributeGroupOpts::Group(opts) => opts,
                        _ => unreachable!(),
                    }),
                    if_contributed_to_groups.as_slice(),
                    after_attempt.as_slice(),
                    pop_error,
                )),
            },
        )
        .collect::<Result<Vec<_>, _>>()
        .map(|v| v.into_iter().flatten().collect())
}

pub fn element_field_deserialize_impl(
    access_expr: &Expr,
    ident_to_expr: impl FnOnce(&FieldIdent) -> Expr,
    FieldWithOpts {
        field_ident,
        field_type,
        options:
            options @ ChildOpts::Value(ValueOpts { .. })
            | options @ ChildOpts::Element(ElementOpts { .. }),
        ..
    }: FieldWithOpts<FieldIdent, ChildOpts>,
    if_next_element_none: &[Stmt],
    finished_element: &[Stmt],
    after_attempt: &[Stmt],
    pop_error: bool,
) -> DeriveResult<Vec<Stmt>> {
    let builder_field_expr = ident_to_expr(&field_ident);
    let temporary_value_ident = Ident::new("__v", Span::call_site());
    let wrapper_ident = Ident::new("__W", Span::call_site());
    let empty_generics: Generics = parse_quote!();

    let wrapper_data = match &options {
        ChildOpts::Element(opts) => {
            let builder =
                opts.to_builder(&field_ident, &wrapper_ident, &empty_generics, &field_type);

            let unwrap_expr = builder.unwrap_expression();

            let deserialize_unwrapper = move |ident: &Ident| {
                let unwrap_expr = unwrap_expr(&parse_quote!(#ident));
                quote! {
                    let mut #ident = #unwrap_expr;
                }
            };

            Some((builder, deserialize_unwrapper))
        }
        _ => None,
    };

    let deserialize_type = wrapper_data
        .as_ref()
        .map(|(a, _)| a.ident)
        .map(|a| parse_quote!(#a))
        .unwrap_or(field_type.clone());

    let deserialize_wrapper_def: Vec<Stmt> = match wrapper_data.as_ref() {
        Some((a, _)) => {
            let def = a.struct_definition();
            let trait_impl = a.deserialize_trait_impl()?;
            parse_quote!(
                #def
                #trait_impl
            )
        }
        None => Vec::new(),
    };

    let deserialize_unwrapper = wrapper_data.as_ref().map(|(_, a)| a);

    let extendable_loop: Option<ExprWhile> = if let ChildOpts::Value(ValueOpts {
        extendable: extendable @ (Extendable::Iterator | Extendable::Single),
        ..
    }) = options
    {
        let loop_temporary_value_ident = Ident::new("__vv", Span::call_site());
        let value_transformer = deserialize_unwrapper
            .cloned()
            .map(|a| (a)(&loop_temporary_value_ident));

        let extendable_value: Expr = if extendable == Extendable::Iterator {
            parse_quote! {
                {
                    let mut #loop_temporary_value_ident = ::core::iter::Iterator::peekable(
                        ::core::iter::IntoIterator::into_iter(#loop_temporary_value_ident)
                    );

                    if ::core::option::Option::is_none(&::core::iter::Peekable::peek(&mut #loop_temporary_value_ident)) {
                        break;
                    }

                    #loop_temporary_value_ident
                }
            }
        } else {
            parse_quote! { [#loop_temporary_value_ident] }
        };

        Some(parse_quote! {
            while let Ok(Some(#loop_temporary_value_ident)) = ::xmlity::de::SeqAccess::next_element_seq::<#deserialize_type>(#access_expr) {
                #value_transformer
                ::core::iter::Extend::extend(&mut #temporary_value_ident, #extendable_value);
            }
        })
    } else {
        None
    };

    let value_transformer = deserialize_unwrapper.map(|a| (a)(&temporary_value_ident));

    let inner = quote!(
        let ::core::option::Option::Some(mut #temporary_value_ident) = #temporary_value_ident else {
            #(#if_next_element_none)*
        };
        #value_transformer
        #extendable_loop
        #builder_field_expr = ::core::option::Option::Some(#temporary_value_ident);

        #(#finished_element)*
    );

    let deserialize_expr: Expr = parse_quote!(
        ::xmlity::de::SeqAccess::next_element_seq::<#deserialize_type>(#access_expr)
    );

    let inner = pop_or_ignore_error(&temporary_value_ident, &deserialize_expr, pop_error, inner);

    let after_attempt = if !pop_error { after_attempt } else { &[] };

    Ok(parse_quote! {
        if ::core::option::Option::is_none(&#builder_field_expr) {
            #(#deserialize_wrapper_def)*

            #inner

            #(#after_attempt)*
        }
    })
}

pub fn pop_or_ignore_error(
    access_ident: &Ident,
    expr: &Expr,
    pop_error: bool,
    inner: impl ToTokens,
) -> proc_macro2::TokenStream {
    if pop_error {
        quote! {
            {
                let mut #access_ident = #expr?;
                #inner
            }
        }
    } else {
        quote! {
            if let ::core::result::Result::Ok(mut #access_ident) = #expr {
                #inner
            }
        }
    }
}

pub fn group_field_contribute_elements(
    access_expr: &Expr,
    ident_to_expr: impl FnOnce(&FieldIdent) -> Expr,
    FieldWithOpts { field_ident, .. }: FieldWithOpts<FieldIdent, GroupOpts>,
    if_contributed_to_groups: &[Stmt],
    after_attempt: &[Stmt],
    pop_error: bool,
) -> DeriveResult<Vec<Stmt>> {
    let builder_field_expr = ident_to_expr(&field_ident);
    let contributed_to_elements_ident = Ident::new("__contributed_to_elements", Span::call_site());

    let deserialize_expr: Expr = parse_quote!(
        ::xmlity::de::DeserializationGroupBuilder::contribute_elements(&mut #builder_field_expr, ::xmlity::de::SeqAccess::sub_access(#access_expr)?)
    );

    let inner = quote! {
        if #contributed_to_elements_ident {
            #(#if_contributed_to_groups)*
        }
    };

    let inner = pop_or_ignore_error(
        &contributed_to_elements_ident,
        &deserialize_expr,
        pop_error,
        inner,
    );

    let after_attempt = if !pop_error { after_attempt } else { &[] };

    Ok(parse_quote! {
        if !::xmlity::de::DeserializationGroupBuilder::elements_done(&#builder_field_expr) {

           #inner
            #(#after_attempt)*

        }
    })
}

pub fn one_stop_field_expression(
    seq_access_ty: &syn::Type,
    seq_access: &syn::Expr,
    visitor_lifetime: &syn::Lifetime,
    de_type: &syn::Type,
    missing_field: &str,
    default_or_else: Option<&Expr>,
    unwrap_function: Option<impl Fn(&Expr) -> Expr>,
) -> syn::Expr {
    let deserialize_expr: Expr = parse_quote!(
        ::xmlity::de::SeqAccess::next_element_seq::<#de_type>(#seq_access)
    );

    let option_value: Expr = parse_quote!(
        ::core::option::Option::flatten(
            ::core::result::Result::ok(
                #deserialize_expr
            )
        )
    );

    let mapped_option_value = if let Some(unwrap_function) = unwrap_function.as_ref() {
        let unwrap_expr = unwrap_function(&parse_quote!(__v));
        parse_quote!(
            ::core::option::Option::map(
                #option_value,
                |__v| #unwrap_expr
            )
        )
    } else {
        option_value
    };

    if let Some(default_or_else) = default_or_else {
        parse_quote!(
            ::core::option::Option::unwrap_or_else(
                #mapped_option_value,
                #default_or_else
            )
        )
    } else {
        let deserialize_none_expr: syn::Expr = parse_quote!(
            <#de_type as ::xmlity::Deserialize<#visitor_lifetime>>::deserialize_seq(
                ::xmlity::types::utils::NoneDeserializer::<
                    <#seq_access_ty as ::xmlity::de::SeqAccess<#visitor_lifetime>>::Error
                >::new()
            )
        );

        let deserialize_none_option_expr: syn::Expr = parse_quote!(
            ::core::result::Result::ok(#deserialize_none_expr)
        );

        let deserialize_none_option_expr = if let Some(unwrap_function) = unwrap_function {
            let unwrap_expr = unwrap_function(&parse_quote!(__v));
            parse_quote!(
                ::core::option::Option::map(
                    #deserialize_none_option_expr,
                    |__v| #unwrap_expr
                )
            )
        } else {
            deserialize_none_option_expr
        };

        parse_quote!(::core::option::Option::ok_or_else(
            ::core::option::Option::or_else(
                #mapped_option_value,
                || {
                    #deserialize_none_option_expr
                },
            ),
            || ::xmlity::de::Error::missing_field(stringify!(#missing_field)),
        )?)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn builder_element_field_visitor<
    F: IntoIterator<Item = FieldWithOpts<FieldIdent, FieldValueGroupOpts>>,
>(
    access_expr: &Expr,
    ident_to_expr: impl Fn(&FieldIdent) -> Expr + Clone,
    fields: F,
    if_next_element_none: Vec<Stmt>,
    finished_element: Vec<Stmt>,
    if_contributed_to_groups: Vec<Stmt>,
    after_attempt: Vec<Stmt>,
    pop_error: bool,
) -> DeriveResult<Vec<Stmt>> {
    fields
        .into_iter()
        .zip(utils::repeat_clone((
            ident_to_expr,
            if_next_element_none,
            finished_element,
            if_contributed_to_groups,
            after_attempt,
        )))
        .map(
            move |(
                var_field,
                (
                    ident_to_expr,
                    if_next_element_none,
                    finished_element,
                    if_contributed_to_groups,
                    after_attempt,
                ),
            )| match &var_field.options {
                FieldValueGroupOpts::Value(_) => element_field_deserialize_impl(
                    access_expr,
                    ident_to_expr,
                    var_field.map_options(|opts| match opts {
                        FieldValueGroupOpts::Value(opts) => opts,
                        _ => unreachable!(),
                    }),
                    if_next_element_none.as_slice(),
                    finished_element.as_slice(),
                    after_attempt.as_slice(),
                    pop_error,
                ),
                FieldValueGroupOpts::Group(_) => group_field_contribute_elements(
                    access_expr,
                    ident_to_expr,
                    var_field.map_options(|opts| match opts {
                        FieldValueGroupOpts::Group(opts) => opts,
                        _ => unreachable!(),
                    }),
                    if_contributed_to_groups.as_slice(),
                    after_attempt.as_slice(),
                    pop_error,
                ),
            },
        )
        .collect::<Result<Vec<_>, _>>()
        .map(|stmts| stmts.into_iter().flatten().collect())
}

pub fn attribute_done_expr(
    FieldWithOpts {
        field_ident,
        options,
        ..
    }: FieldWithOpts<FieldIdent, FieldAttributeGroupOpts>,
    ident_to_expr: impl FnOnce(&FieldIdent) -> Expr,
) -> Expr {
    let builder_field_expr = ident_to_expr(&field_ident);
    match options {
        FieldAttributeGroupOpts::Attribute(_) => parse_quote! {
            ::core::option::Option::is_some( #builder_field_expr)
        },
        FieldAttributeGroupOpts::Group(_) => parse_quote! {
            ::xmlity::de::DeserializationGroupBuilder::attributes_done(#builder_field_expr)
        },
    }
}

pub fn all_attributes_done_expr(
    fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, FieldAttributeGroupOpts>>,
    ident_to_expr: impl Fn(&FieldIdent) -> Expr,
) -> Expr {
    let conditions = fields
        .into_iter()
        .map(|field| attribute_done_expr(field, &ident_to_expr));

    let conditions = quote! {
        #(#conditions)&&*
    };

    if conditions.is_empty() {
        parse_quote! {true}
    } else {
        parse_quote! {#conditions}
    }
}

pub fn element_done_expr(
    FieldWithOpts {
        field_ident,
        options,
        ..
    }: FieldWithOpts<FieldIdent, FieldValueGroupOpts>,
    ident_to_expr: impl FnOnce(&FieldIdent) -> Expr,
) -> Expr {
    let field_expr = ident_to_expr(&field_ident);
    match options {
        FieldValueGroupOpts::Value(_) => parse_quote! {
            ::core::option::Option::is_some(#field_expr)
        },
        FieldValueGroupOpts::Group(_) => parse_quote! {
            ::xmlity::de::DeserializationGroupBuilder::elements_done(#field_expr)
        },
    }
}

pub fn all_elements_done_expr(
    fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, FieldValueGroupOpts>>,
    ident_to_expr: impl Fn(&FieldIdent) -> Expr,
) -> Expr {
    let conditions = fields
        .into_iter()
        .map(|field| element_done_expr(field, &ident_to_expr));

    let conditions = quote! {
        #(#conditions)&&*
    };

    if conditions.is_empty() {
        parse_quote! {true}
    } else {
        parse_quote! {#conditions}
    }
}

pub fn fields(
    ast: &syn::DeriveInput,
) -> Result<impl IntoIterator<Item = FieldWithOpts<FieldIdent, FieldOpts>>, DeriveError> {
    let data_struct = match ast.data {
        syn::Data::Struct(ref data_struct) => data_struct,
        _ => unreachable!(),
    };

    Ok(match &data_struct.fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .map(|f| {
                let field_ident = f.ident.clone().expect("Named struct");

                DeriveResult::Ok(FieldWithOpts {
                    field_ident: FieldIdent::Named(field_ident),
                    options: FieldOpts::from_field(f)?,
                    field_type: f.ty.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| {
                DeriveResult::Ok(FieldWithOpts {
                    field_ident: FieldIdent::Indexed(syn::Index::from(i)),
                    options: FieldOpts::from_field(f)?,
                    field_type: f.ty.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => Vec::new(),
    })
}

pub fn element_fields(
    ast: &syn::DeriveInput,
) -> Result<impl IntoIterator<Item = FieldWithOpts<FieldIdent, ChildOpts>> + use<'_>, DeriveError> {
    Ok(fields(ast)?.into_iter().filter_map(|field| {
        field.map_options_opt(|opt| match opt {
            FieldOpts::Value(opts) => Some(opts),
            _ => None,
        })
    }))
}

pub fn attribute_fields(
    ast: &syn::DeriveInput,
) -> Result<impl IntoIterator<Item = FieldWithOpts<FieldIdent, AttributeOpts>> + use<'_>, DeriveError>
{
    Ok(fields(ast)?.into_iter().filter_map(|field| {
        field.map_options_opt(|opt| match opt {
            FieldOpts::Attribute(opts) => Some(opts),
            _ => None,
        })
    }))
}

pub fn group_fields(
    ast: &syn::DeriveInput,
) -> Result<impl IntoIterator<Item = FieldWithOpts<FieldIdent, GroupOpts>> + use<'_>, DeriveError> {
    Ok(fields(ast)?.into_iter().filter_map(|field| {
        field.clone().map_options_opt(|opt| match opt {
            FieldOpts::Group(opts) => Some(opts),
            _ => None,
        })
    }))
}

pub fn attribute_group_fields(
    ast: &syn::DeriveInput,
) -> Result<
    impl IntoIterator<Item = FieldWithOpts<FieldIdent, FieldAttributeGroupOpts>> + use<'_>,
    DeriveError,
> {
    Ok(fields(ast)?.into_iter().filter_map(|field| {
        field.clone().map_options_opt(|opt| match opt {
            FieldOpts::Attribute(opts) => Some(FieldAttributeGroupOpts::Attribute(opts)),
            FieldOpts::Group(opts) => Some(FieldAttributeGroupOpts::Group(opts)),
            FieldOpts::Value(_) => None,
        })
    }))
}

pub fn element_group_fields(
    ast: &syn::DeriveInput,
) -> Result<
    impl IntoIterator<Item = FieldWithOpts<FieldIdent, FieldValueGroupOpts>> + use<'_>,
    DeriveError,
> {
    Ok(fields(ast)?.into_iter().filter_map(|field| {
        field.clone().map_options_opt(|opt| match opt {
            FieldOpts::Value(opts) => Some(FieldValueGroupOpts::Value(opts)),
            FieldOpts::Group(opts) => Some(FieldValueGroupOpts::Group(opts)),
            FieldOpts::Attribute(_) => None,
        })
    }))
}

pub fn deserialize_option_value_expr(
    field_type: &syn::Type,
    field_ident: &Expr,
    default_or_else: Option<Expr>,
    should_try_none: bool,
    visitor_lifetime: &syn::Lifetime,
    error_type: &syn::Type,
    missing_field: &str,
) -> Expr {
    if let Some(default_or_else) = default_or_else {
        parse_quote! {
            ::core::option::Option::unwrap_or_else(#field_ident, #default_or_else)
        }
    } else if should_try_none {
        parse_quote! {
            ::core::result::Result::map_err(
                ::core::option::Option::map_or_else(
                    #field_ident,
                    || <#field_type as ::xmlity::Deserialize<#visitor_lifetime>>::deserialize_seq(
                        ::xmlity::types::utils::NoneDeserializer::<#error_type>::new(),
                    ),
                    |__v| ::core::result::Result::Ok(__v)
                ),
                |_|  ::xmlity::de::Error::missing_field(stringify!(#missing_field))
            )?
        }
    } else {
        parse_quote! {
            ::core::option::Option::ok_or(#field_ident, ::xmlity::de::Error::missing_field(stringify!(#missing_field)))?
        }
    }
}
