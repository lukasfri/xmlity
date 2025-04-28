mod deserialization_group;
mod deserialize;
use common::DeserializeBuilderExt;
pub use deserialization_group::DeriveDeserializationGroup;
pub use deserialize::DeriveDeserialize;
use deserialize::SingleChildDeserializeElementBuilder;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse_quote, Expr, ExprWhile, Generics, Ident, ItemStruct, Stmt, Type, Visibility};

use crate::{
    options::{
        structs::fields::{
            AttributeOpts, ChildOpts, ElementOpts, FieldAttributeGroupOpts, FieldOpts,
            FieldValueGroupOpts, GroupOpts, ValueOpts,
        },
        WithExpandedNameExt,
    },
    utils::{self},
    DeriveError, DeriveResult, DeserializeField, FieldIdent,
};

mod common;

#[derive(Clone, Copy)]
enum StructType {
    Named,
    Unnamed,
    Unit,
}

#[derive(Clone)]
enum StructTypeWithFields<N, U> {
    Named(N),
    Unnamed(U),
    Unit,
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
    let mut fields = fields.into_iter();
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
        StructType::Unit => {
            assert!(fields.next().is_none(), "unit structs cannot have fields");
            quote! { #ident }
        }
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

fn unit_struct_definition_expr<I: ToTokens>(
    ident: I,
    generics: Option<&syn::Generics>,
    visibility: &Visibility,
) -> proc_macro2::TokenStream {
    quote! {
        #visibility struct #ident #generics;
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
        StructType::Unit => unit_struct_definition_expr(ident, generics, visibility),
    }
}

fn attribute_field_deserialize_impl(
    access_ident: &Ident,
    builder_field_ident_prefix: impl ToTokens,
    DeserializeField {
        field_ident,
        field_type,
        ..
    }: DeserializeField<FieldIdent, AttributeOpts>,
    if_next_attribute_none: &[Stmt],
    finished_attribute: &[Stmt],
    after_attempt: &[Stmt],
    pop_error: bool,
) -> Vec<Stmt> {
    let temporary_value_ident = Ident::new("__v", Span::call_site());

    let builder_field_ident = field_ident.to_named_ident();

    let inner = quote! {
        let ::core::option::Option::Some(#temporary_value_ident) = #temporary_value_ident else {
            #(#if_next_attribute_none)*
        };
        #builder_field_ident_prefix #builder_field_ident = ::core::option::Option::Some(#temporary_value_ident);
        #(#finished_attribute)*
    };

    let deserialize_expr: Expr = parse_quote!(
        ::xmlity::de::AttributesAccess::next_attribute::<#field_type>(&mut #access_ident)
    );

    let inner = pop_or_ignore_error(&temporary_value_ident, &deserialize_expr, pop_error, inner);

    let after_attempt = if !pop_error { after_attempt } else { &[] };

    parse_quote! {
        if ::core::option::Option::is_none(&#builder_field_ident_prefix #builder_field_ident) {
            #inner
            #(#after_attempt)*
        }
    }
}

fn group_field_contribute_attributes(
    access_ident: &Ident,
    builder_field_ident_prefix: impl ToTokens,
    DeserializeField { field_ident, .. }: DeserializeField<FieldIdent, GroupOpts>,
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
fn builder_attribute_field_visitor<
    F: IntoIterator<Item = DeserializeField<FieldIdent, FieldAttributeGroupOpts>>,
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
    fields
        .into_iter()
        .zip(utils::repeat_clone((
            builder_field_ident_prefix,
            if_next_attribute_none,
            finished_attribute,
            if_contributed_to_groups,
            after_attempt,
        )))
        .flat_map(
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
                FieldAttributeGroupOpts::Group(_) => group_field_contribute_attributes(
                    access_ident,
                    builder_field_ident_prefix,
                    var_field.map_options(|opts| match opts {
                        FieldAttributeGroupOpts::Group(opts) => opts,
                        _ => unreachable!(),
                    }),
                    if_contributed_to_groups.as_slice(),
                    after_attempt.as_slice(),
                    pop_error,
                ),
            },
        )
        .collect::<Vec<_>>()
}

fn element_field_deserialize_impl(
    access_ident: &Ident,
    builder_field_ident_prefix: impl ToTokens,
    DeserializeField {
        field_ident,
        field_type,
        options:
            options @ ChildOpts::Value(ValueOpts { extendable, .. })
            | options @ ChildOpts::Element(ElementOpts { extendable, .. }),
        ..
    }: DeserializeField<FieldIdent, ChildOpts>,
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
        ChildOpts::Element(opts) => {
            let wrapper_def: ItemStruct = parse_quote! {
                struct #wrapper_ident {
                    __value: #field_type,
                }
            };
            let builder = SingleChildDeserializeElementBuilder {
                ident: &wrapper_ident,
                generics: &empty_generics,
                required_expanded_name: Some(
                    opts.expanded_name(field_ident.to_named_ident().to_string().as_str())
                        .into_owned(),
                ),
                item_type: &field_type,
                extendable,
            };

            let deserialize_unwrapper = |ident: &Ident| {
                quote! {
                    let mut #ident = #ident.__value;
                }
            };

            Some((wrapper_def, builder, deserialize_unwrapper))
        }
        _ => None,
    };

    let deserialize_type = wrapper_data
        .as_ref()
        .map(|(_, a, _)| a.ident)
        .map(|a| parse_quote!(#a))
        .unwrap_or(field_type.clone());

    let deserialize_wrapper_def: Vec<Stmt> = match wrapper_data.as_ref() {
        Some((a, b, _)) => {
            let b = b.deserialize_trait_impl()?;
            parse_quote!(
                #a
                #b
            )
        }
        None => Vec::new(),
    };

    let deserialize_unwrapper = wrapper_data.as_ref().map(|(_, _, a)| a);

    let extendable_loop: Option<ExprWhile> = if extendable {
        let loop_temporary_value_ident = Ident::new("__vv", Span::call_site());
        let value_transformer = deserialize_unwrapper
            .copied()
            .map(|a| (a)(&loop_temporary_value_ident));
        Some(parse_quote! {
            while let Some(#loop_temporary_value_ident) = ::xmlity::de::SeqAccess::next_element_seq::<#deserialize_type>(&mut #access_ident)? {
                #value_transformer
                ::core::iter::Extend::extend(&mut #temporary_value_ident, [#loop_temporary_value_ident]);
            }
        })
    } else {
        None
    };

    let deserialize_expr: Expr = parse_quote!(
        ::xmlity::de::SeqAccess::next_element_seq::<#deserialize_type>(&mut #access_ident)
    );

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

fn pop_or_ignore_error(
    access_ident: &Ident,
    expr: &Expr,
    pop_error: bool,
    inner: impl ToTokens,
) -> proc_macro2::TokenStream {
    if pop_error {
        quote! {
            if let ::core::result::Result::Ok(mut #access_ident) = #expr {
                #inner
            }
        }
    } else {
        quote! {
            {
                let mut #access_ident = #expr?;
                #inner
            }
        }
    }
}

fn group_field_contribute_elements(
    access_ident: &Ident,
    builder_field_ident_prefix: impl ToTokens,
    DeserializeField { field_ident, .. }: DeserializeField<FieldIdent, GroupOpts>,
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
fn builder_element_field_visitor<
    F: IntoIterator<Item = DeserializeField<FieldIdent, FieldValueGroupOpts>>,
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

fn attribute_done_expr(
    DeserializeField {
        field_ident,
        options,
        ..
    }: DeserializeField<FieldIdent, FieldAttributeGroupOpts>,
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

fn all_attributes_done_expr(
    fields: impl IntoIterator<Item = DeserializeField<FieldIdent, FieldAttributeGroupOpts>>,

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

fn element_done_expr(
    DeserializeField {
        field_ident,
        options,
        ..
    }: DeserializeField<FieldIdent, FieldValueGroupOpts>,
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

fn all_elements_done_expr(
    fields: impl IntoIterator<Item = DeserializeField<FieldIdent, FieldValueGroupOpts>>,
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
) -> Result<impl IntoIterator<Item = DeserializeField<FieldIdent, FieldOpts>>, DeriveError> {
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

                DeriveResult::Ok(DeserializeField {
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
                DeriveResult::Ok(DeserializeField {
                    field_ident: FieldIdent::Indexed(syn::Index::from(i)),
                    options: FieldOpts::from_field(f)?,
                    field_type: f.ty.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => unreachable!(),
    })
}

fn element_fields(
    ast: &syn::DeriveInput,
) -> Result<impl IntoIterator<Item = DeserializeField<FieldIdent, ChildOpts>> + use<'_>, DeriveError>
{
    Ok(fields(ast)?.into_iter().filter_map(|field| {
        field.map_options_opt(|opt| match opt {
            FieldOpts::Value(opts) => Some(opts),
            _ => None,
        })
    }))
}

fn attribute_fields(
    ast: &syn::DeriveInput,
) -> Result<
    impl IntoIterator<Item = DeserializeField<FieldIdent, AttributeOpts>> + use<'_>,
    DeriveError,
> {
    Ok(fields(ast)?.into_iter().filter_map(|field| {
        field.map_options_opt(|opt| match opt {
            FieldOpts::Attribute(opts) => Some(opts),
            _ => None,
        })
    }))
}

fn group_fields(
    ast: &syn::DeriveInput,
) -> Result<impl IntoIterator<Item = DeserializeField<FieldIdent, GroupOpts>> + use<'_>, DeriveError>
{
    Ok(fields(ast)?.into_iter().filter_map(|field| {
        field.clone().map_options_opt(|opt| match opt {
            FieldOpts::Group(opts) => Some(opts),
            _ => None,
        })
    }))
}

fn attribute_group_fields(
    ast: &syn::DeriveInput,
) -> Result<
    impl IntoIterator<Item = DeserializeField<FieldIdent, FieldAttributeGroupOpts>> + use<'_>,
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

fn element_group_fields(
    ast: &syn::DeriveInput,
) -> Result<
    impl IntoIterator<Item = DeserializeField<FieldIdent, FieldValueGroupOpts>> + use<'_>,
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
