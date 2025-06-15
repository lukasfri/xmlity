use syn::{parse_quote, spanned::Spanned, Expr, Ident, Stmt};

use crate::{
    common::FieldIdent,
    de::common::{builder_element_field_visitor, deserialize_option_value_expr},
    derive::DeriveResult,
    options::{
        records::fields::{ChildOpts, FieldValueGroupOpts},
        AllowUnknown, ElementOrder, FieldWithOpts, IgnoreWhitespace,
    },
};

pub struct SeqLoopAccessor {
    allow_unknown_children: AllowUnknown,
    order: ElementOrder,
    ignore_whitespace: IgnoreWhitespace,
}

impl SeqLoopAccessor {
    pub fn new(
        allow_unknown_children: AllowUnknown,
        order: ElementOrder,
        ignore_whitespace: IgnoreWhitespace,
    ) -> Self {
        Self {
            allow_unknown_children,
            order,
            ignore_whitespace,
        }
    }

    pub fn field_definitions<
        F: IntoIterator<Item = FieldWithOpts<FieldIdent, FieldValueGroupOpts>>,
    >(
        &self,
        fields: F,
    ) -> DeriveResult<Vec<Stmt>> {
        fields
            .into_iter()
            .map::<DeriveResult<Stmt>, _>(
                |FieldWithOpts {
                     field_ident,
                     field_type,
                     options,
                     ..
                 }| {
                    let builder_initializer: syn::Expr = match options {
                        FieldValueGroupOpts::Value(_) => parse_quote! {
                             ::core::option::Option::<#field_type>::None
                        },
                        FieldValueGroupOpts::Group(_) => parse_quote! {
                            <#field_type as ::xmlity::de::DeserializationGroup>::builder()
                        },
                    };

                    let builder_field_ident = field_ident.to_named_ident();
                    Ok(parse_quote! {
                        let mut #builder_field_ident = #builder_initializer;
                    })
                },
            )
            .collect()
    }

    pub fn access_loop<F: IntoIterator<Item = FieldWithOpts<FieldIdent, FieldValueGroupOpts>>>(
        &self,
        fields: F,
        access_expr: &Expr,
    ) -> DeriveResult<Vec<Stmt>> {
        let Self {
            allow_unknown_children,
            order,

            ignore_whitespace,
        } = self;

        let field_visits = builder_element_field_visitor(
            access_expr,
            |field| {
                let field = field.to_named_ident();
                parse_quote!(#field)
            },
            fields,
            parse_quote! {break;},
            match order {
                ElementOrder::Strict => parse_quote! {break;},
                ElementOrder::None => parse_quote! {continue;},
            },
            parse_quote! {continue;},
            parse_quote! {},
            false,
        )?;

        let ignore_whitespace_expression: Vec<Stmt> = match ignore_whitespace {
            IgnoreWhitespace::Any => {
                parse_quote! {
                    if let Ok(Some(_)) = ::xmlity::de::SeqAccess::next_element::<::xmlity::types::utils::Whitespace>(#access_expr) {
                        continue;
                    }
                }
            }
            IgnoreWhitespace::None => {
                vec![]
            }
        };

        let skip_unknown: Vec<Stmt> = match allow_unknown_children {
            AllowUnknown::Any => {
                let skip_ident = Ident::new("__skip", access_expr.span());
                parse_quote! {
                    let #skip_ident = ::xmlity::de::SeqAccess::next_element::<::xmlity::types::utils::IgnoredAny>(#access_expr).unwrap_or(None);
                    if ::core::option::Option::is_none(&#skip_ident) {
                        break;
                    }
                    continue;
                }
            }
            AllowUnknown::AtEnd => {
                //Ignore whatever is left
                parse_quote! {
                    break;
                }
            }
            AllowUnknown::None => {
                //Check that nothing is left
                let skip_ident = Ident::new("__skip", access_expr.span());
                parse_quote! {
                    let #skip_ident = ::xmlity::de::SeqAccess::next_element::<::xmlity::types::utils::IgnoredAny>(#access_expr).unwrap_or(None);
                    if ::core::option::Option::is_none(&#skip_ident) {
                        break;
                    }

                    return Err(::xmlity::de::Error::unknown_child());
                }
            }
        };

        match order {
            ElementOrder::Strict => field_visits
                .into_iter()
                .map(|field_visit| {
                    Ok(parse_quote! {
                        loop {
                            #(#ignore_whitespace_expression)*
                            #field_visit
                            #(#skip_unknown)*
                        }
                    })
                })
                .collect(),
            ElementOrder::None => Ok(parse_quote! {
                loop {
                    #(#ignore_whitespace_expression)*
                    #(#field_visits)*
                    #(#skip_unknown)*
                }
            }),
        }
    }

    pub fn value_expressions<
        F: IntoIterator<Item = FieldWithOpts<FieldIdent, FieldValueGroupOpts>>,
    >(
        &self,
        fields: F,
        visitor_lifetime: &syn::Lifetime,
        error_type: &syn::Type,
    ) -> DeriveResult<Vec<(FieldIdent, Expr)>> {
        fields
            .into_iter()
            .map(
                |FieldWithOpts { field_ident, field_type, options }| {
                    let builder_field_ident = field_ident.to_named_ident();

                    let expr = match options {
                        FieldValueGroupOpts::Value(opts) => {

                            deserialize_option_value_expr(
                                &field_type,
                                &parse_quote!(#builder_field_ident),
                                opts.default_or_else(),
                                matches!(opts, ChildOpts::Value(_)),
                                visitor_lifetime,
                                error_type,
                                &field_ident.to_string(),
                            )
                        },
                        FieldValueGroupOpts::Group(_) => {
                            parse_quote! {
                                ::xmlity::de::DeserializationGroupBuilder::finish::<#error_type>(#builder_field_ident)?
                            }
                        },
                    };

                    Ok((field_ident, expr))
                },
            ).collect::<Result<Vec<_>, _>>()
    }
}
