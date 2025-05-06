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
use syn::spanned::Spanned;

use crate::options::ElementOrder;

use super::deserialize::{DeserializeSingleChildElementBuilder, SimpleDeserializeAttributeBuilder};

pub struct SeqVisitLoop<
    'a,
    F: IntoIterator<Item = FieldWithOpts<FieldIdent, FieldValueGroupOpts>> + Clone,
> {
    seq_access_ident: &'a Ident,
    allow_unknown_children: bool,
    order: ElementOrder,
    fields: F,
}

impl<'a, F: IntoIterator<Item = FieldWithOpts<FieldIdent, FieldValueGroupOpts>> + Clone>
    SeqVisitLoop<'a, F>
{
    pub fn new(
        seq_access_ident: &'a Ident,
        allow_unknown_children: bool,
        order: ElementOrder,
        fields: F,
    ) -> Self {
        Self {
            seq_access_ident,
            allow_unknown_children,
            order,
            fields,
        }
    }

    pub fn field_storage(&self) -> proc_macro2::TokenStream {
        quote! {}
    }

    pub fn access_loop(&self) -> DeriveResult<Vec<Stmt>> {
        let Self {
            seq_access_ident: access_ident,
            allow_unknown_children,
            order,
            fields,
        } = self;

        let pop_error = matches!(order, ElementOrder::Loose);

        let field_visits = builder_element_field_visitor(
            access_ident,
            quote! {},
            fields.clone(),
            parse_quote! {break;},
            match order {
                ElementOrder::Loose => parse_quote! {break;},
                ElementOrder::None => parse_quote! {continue;},
            },
            parse_quote! {continue;},
            parse_quote! {},
            pop_error,
        )?;

        match order {
            ElementOrder::Loose => field_visits.into_iter().zip(fields.clone()).map(|(field_visit, field)| {
                let skip_unknown: Vec<Stmt> = if *allow_unknown_children {
                    let skip_ident = Ident::new("__skip", access_ident.span());
                    parse_quote! {
                        let #skip_ident = ::xmlity::de::SeqAccess::next_element::<::xmlity::types::utils::IgnoredAny>(&mut #access_ident).unwrap_or(None);
                        if ::core::option::Option::is_none(&#skip_ident) {
                            break;
                        }
                        continue;
                    }
                } else {
                    let condition = element_done_expr(field, quote! {});

                    parse_quote! {
                        if #condition {
                            break;
                        } else {
                            return ::core::result::Result::Err(::xmlity::de::Error::unknown_child());
                        }
                    }
                };

                Ok(parse_quote! {
                    loop {
                        #field_visit
                        #(#skip_unknown)*
                    }
                })
            }).collect(),
            ElementOrder::None => {
                let skip_unknown: Vec<Stmt> = if *allow_unknown_children {
                    let skip_ident = Ident::new("__skip", access_ident.span());
                    parse_quote! {
                        let #skip_ident = ::xmlity::de::SeqAccess::next_element::<::xmlity::types::utils::IgnoredAny>(&mut #access_ident).unwrap_or(None);
                        if ::core::option::Option::is_none(&#skip_ident) {
                            break
                        }
                    }
                } else {
                    let all_some_condition = all_elements_done_expr(fields.clone(), quote! {});

                    parse_quote! {
                        if #all_some_condition {
                            break;
                        } else {
                            return ::core::result::Result::Err(::xmlity::de::Error::unknown_child());
                        }
                    }
                };

                Ok(parse_quote! {
                    loop {
                        #(#field_visits)*
                        #(#skip_unknown)*
                    }
                })
            },
        }
    }
}

fn attribute_field_deserialize_impl(
    access_ident: &Ident,
    builder_field_ident_prefix: impl ToTokens,
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
    let builder_field_ident = field_ident.to_named_ident();
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
        #builder_field_ident_prefix #builder_field_ident = ::core::option::Option::Some(#temporary_value_ident);
        #(#finished_attribute)*
    };
    let deserialize_expr: Expr = parse_quote!(
        ::xmlity::de::AttributesAccess::next_attribute::<#deserialize_type>(&mut #access_ident)
    );

    let inner = pop_or_ignore_error(&temporary_value_ident, &deserialize_expr, pop_error, inner);

    let after_attempt = if !pop_error { after_attempt } else { &[] };

    Ok(parse_quote! {
        if ::core::option::Option::is_none(&#builder_field_ident_prefix #builder_field_ident) {
            #(#deserialize_wrapper_def)*

            #inner

            #(#after_attempt)*
        }
    })
}

fn group_field_contribute_attributes(
    access_ident: &Ident,
    builder_field_ident_prefix: impl ToTokens,
    FieldWithOpts { field_ident, .. }: FieldWithOpts<FieldIdent, GroupOpts>,
    if_contributed_to_groups: &[Stmt],
    after_attempt: &[Stmt],
    pop_error: bool,
) -> Vec<Stmt> {
    let builder_field_ident = field_ident.to_named_ident();
    let contributed_to_attributes_ident =
        Ident::new("__contributed_to_attributes", Span::call_site());

    let deserialize_expr: Expr = parse_quote!(
        ::xmlity::de::DeserializationGroupBuilder::contribute_attributes(&mut #builder_field_ident_prefix #builder_field_ident, ::xmlity::de::AttributesAccess::sub_access(&mut #access_ident)?)
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
        if !::xmlity::de::DeserializationGroupBuilder::attributes_done(&#builder_field_ident_prefix #builder_field_ident) {
            #inner
            #(#after_attempt)*
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn builder_attribute_field_visitor<
    F: IntoIterator<Item = FieldWithOpts<FieldIdent, FieldAttributeGroupOpts>>,
>(
    access_ident: &Ident,
    builder_field_ident_prefix: proc_macro2::TokenStream,
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
                FieldAttributeGroupOpts::Attribute(_) => attribute_field_deserialize_impl(
                    access_ident,
                    builder_field_ident_prefix,
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
                    access_ident,
                    builder_field_ident_prefix,
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
    access_ident: &Ident,
    builder_field_ident_prefix: impl ToTokens,
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
    let builder_field_ident = field_ident.to_named_ident();
    let temporary_value_ident = Ident::new("__v", Span::call_site());
    let wrapper_ident = Ident::new("__W", Span::call_site());
    let empty_generics: Generics = parse_quote!();

    let wrapper_data = match &options {
        ChildOpts::Element(opts @ ElementOpts { extendable, .. }) => {
            let builder = DeserializeSingleChildElementBuilder {
                ident: &wrapper_ident,
                generics: &empty_generics,
                required_expanded_name: Some(
                    opts.expanded_name(field_ident.to_named_ident().to_string().as_str())
                        .into_owned(),
                ),
                item_type: &field_type,
                extendable: *extendable,
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
            .copied()
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
            while let Some(#loop_temporary_value_ident) = ::xmlity::de::SeqAccess::next_element_seq::<#deserialize_type>(&mut #access_ident)? {
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
        #builder_field_ident_prefix #builder_field_ident = ::core::option::Option::Some(#temporary_value_ident);

        #(#finished_element)*
    );

    let deserialize_expr: Expr = parse_quote!(
        ::xmlity::de::SeqAccess::next_element_seq::<#deserialize_type>(&mut #access_ident)
    );

    let inner = pop_or_ignore_error(&temporary_value_ident, &deserialize_expr, pop_error, inner);

    let after_attempt = if !pop_error { after_attempt } else { &[] };

    Ok(parse_quote! {
        if ::core::option::Option::is_none(&#builder_field_ident_prefix #builder_field_ident) {
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
    access_ident: &Ident,
    builder_field_ident_prefix: impl ToTokens,
    FieldWithOpts { field_ident, .. }: FieldWithOpts<FieldIdent, GroupOpts>,
    if_contributed_to_groups: &[Stmt],
    after_attempt: &[Stmt],
    pop_error: bool,
) -> DeriveResult<Vec<Stmt>> {
    let builder_field_ident = field_ident.to_named_ident();
    let contributed_to_elements_ident = Ident::new("__contributed_to_elements", Span::call_site());

    let deserialize_expr: Expr = parse_quote!(
        ::xmlity::de::DeserializationGroupBuilder::contribute_elements(&mut #builder_field_ident_prefix #builder_field_ident, ::xmlity::de::SeqAccess::sub_access(&mut #access_ident)?)
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
        if !::xmlity::de::DeserializationGroupBuilder::elements_done(&#builder_field_ident_prefix #builder_field_ident) {

           #inner
            #(#after_attempt)*

        }
    })
}

#[allow(clippy::too_many_arguments)]
pub fn builder_element_field_visitor<
    F: IntoIterator<Item = FieldWithOpts<FieldIdent, FieldValueGroupOpts>>,
>(
    access_ident: &Ident,
    builder_field_ident_prefix: proc_macro2::TokenStream,
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
                FieldValueGroupOpts::Value(_) => element_field_deserialize_impl(
                    access_ident,
                    builder_field_ident_prefix,
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
                    access_ident,
                    builder_field_ident_prefix,
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
    builder_field_ident_prefix: impl ToTokens,
) -> Expr {
    let builder_field_ident = field_ident.to_named_ident();
    match options {
        FieldAttributeGroupOpts::Attribute(_) => parse_quote! {
            ::core::option::Option::is_some(&#builder_field_ident_prefix #builder_field_ident)
        },
        FieldAttributeGroupOpts::Group(_) => parse_quote! {
            ::xmlity::de::DeserializationGroupBuilder::attributes_done(&#builder_field_ident_prefix #builder_field_ident)
        },
    }
}

pub fn all_attributes_done_expr(
    fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, FieldAttributeGroupOpts>>,

    builder_field_ident_prefix: impl ToTokens,
) -> Expr {
    let conditions = fields
        .into_iter()
        .map(|field| attribute_done_expr(field, &builder_field_ident_prefix));

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
    builder_field_ident_prefix: impl ToTokens,
) -> Expr {
    let builder_field_ident = field_ident.to_named_ident();
    match options {
        FieldValueGroupOpts::Value(_) => parse_quote! {
            ::core::option::Option::is_some(&#builder_field_ident_prefix #builder_field_ident)
        },
        FieldValueGroupOpts::Group(_) => parse_quote! {
            ::xmlity::de::DeserializationGroupBuilder::elements_done(&#builder_field_ident_prefix #builder_field_ident)
        },
    }
}

pub fn all_elements_done_expr(
    fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, FieldValueGroupOpts>>,
    builder_field_ident_prefix: impl ToTokens,
) -> Expr {
    let conditions = fields
        .into_iter()
        .map(|field| element_done_expr(field, &builder_field_ident_prefix));

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
        _ => unreachable!(),
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
