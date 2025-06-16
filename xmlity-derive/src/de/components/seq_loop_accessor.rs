use proc_macro2::Span;
use syn::{parse_quote, spanned::Spanned, Expr, Generics, Ident, Lifetime, Stmt, Type};

use crate::{
    common::FieldIdent,
    de::{builders::DeserializeBuilderExt, common::{
        builder_element_field_visitor, deserialize_option_value_expr, one_stop_field_expression,
    }},
    derive::{DeriveError, DeriveResult},
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
        seq_access: &Expr,
        seq_access_ty: &Type,
        visitor_lifetime: &Lifetime,
    ) -> DeriveResult<Vec<Stmt>> {
        let Self {
            allow_unknown_children,
            order,

            ignore_whitespace,
        } = self;

        let whitespace_ty: syn::Type = parse_quote! {::xmlity::types::utils::Whitespace};

        let ignore_whitespace_expression: Vec<Stmt> = match ignore_whitespace {
            IgnoreWhitespace::Any => {
                parse_quote! {
                    if let Ok(Some(_)) = ::xmlity::de::SeqAccess::next_element::<#whitespace_ty>(#seq_access) {
                        continue;
                    }
                }
            }
            IgnoreWhitespace::None => {
                vec![]
            }
        };

        let ignored_any_ty: syn::Type = parse_quote! {::xmlity::types::utils::IgnoredAny};

        match order {
            ElementOrder::Strict => {
                let end_check: Vec<Stmt> = match allow_unknown_children {
                    AllowUnknown::Any => {
                        return Err(DeriveError::custom(
                            "An unknown element in any position is not allowed in strict order",
                        ))
                    }
                    AllowUnknown::AtEnd => {
                        //Ignore whatever is left
                        Vec::new()
                    }
                    AllowUnknown::None => {
                        //Check that nothing is left
                        parse_quote! {
                            if let Ok(Some(_)) = ::xmlity::de::SeqAccess::next_element::<#ignored_any_ty>(#seq_access)  {
                                return Err(::xmlity::de::Error::custom("Unexpected element at end of sequence."));
                            }
                        }
                    }
                };

                let field_visits = fields.into_iter().map::<DeriveResult<(_, syn::Expr, Vec<Stmt>)>, _>(|f| {
                    let (condition, deserialize_stmts) = match &f.options {
                        FieldValueGroupOpts::Value(child_opts) => {
                            let wrapper_ident = Ident::new("__W", Span::call_site());
                            let empty_generics: Generics = parse_quote!();

                            let (prefix, wrapped_de_type, unwrap_function) = match child_opts {
                                ChildOpts::Value(_) => (Vec::new(), None, None),
                                ChildOpts::Element(element_opts) => {
                                    let builder = element_opts.to_builder(
                                        &f.field_ident,
                                        &wrapper_ident,
                                        &empty_generics,
                                        &f.field_type,
                                    );


                                    let deserialize_wrapper_def: Vec<Stmt> = {
                                        let def = builder.struct_definition();
                                        let trait_impl = builder.deserialize_trait_impl()?;
                                        parse_quote!(
                                            #def
                                            #trait_impl
                                        )
                                    };

                                    let struct_type: Type = parse_quote!(#wrapper_ident);
                                    let unwrap_function = builder.unwrap_expression();

                                    (
                                        deserialize_wrapper_def,
                                        Some(struct_type),
                                        Some(unwrap_function),
                                    )
                                }
                            };

                            let value_expr = one_stop_field_expression(
                                seq_access_ty,
                                seq_access,
                                visitor_lifetime,
                                wrapped_de_type.as_ref().unwrap_or(&f.field_type),
                                f.field_ident.to_string().as_str(),
                                child_opts.default_or_else().as_ref(),
                                unwrap_function,
                            );

                            let builder_ident = f.field_ident.to_named_ident();

                            let condition: syn::Expr = parse_quote!(
                                ::core::option::Option::is_none(&#builder_ident)
                            );

                            let deserialize_stmts = parse_quote!(
                                #(#prefix)*
                                #builder_ident = ::core::option::Option::Some(#value_expr);
                            );

                            (condition, deserialize_stmts)
                        }
                        FieldValueGroupOpts::Group(_) => {

                            let builder_ident = f.field_ident.to_named_ident();

                            let condition: syn::Expr = parse_quote!(
                                ::xmlity::de::DeserializationGroupBuilder::elements_done(&#builder_ident)
                            );
                            
                            let deserialize_expr: Expr = parse_quote!(
                                ::xmlity::de::DeserializationGroupBuilder::contribute_elements(&mut #builder_ident, ::xmlity::de::SeqAccess::sub_access(#seq_access)?)?
                            );

                            let deserialize_stmts = parse_quote! {
                                if !#deserialize_expr {
                                    return ::core::result::Result::Err(::xmlity::de::Error::custom("Failed to deserialize group"));
                                }
                            };

                            (condition, deserialize_stmts)
                        }
                    };

                    Ok((f, condition, deserialize_stmts))
                }).collect::<Result<Vec<_>, _>>()?;

                let if_statements = field_visits.into_iter().map(|(_f, condition, deserialize_stmts)| {
                    parse_quote!(
                        if #condition {
                            #(#deserialize_stmts)*
                        }
                    )
                });

                // Bind the if_statements together to if else if else if else
                let if_statements: Option<proc_macro2::TokenStream> =
                    if_statements.reduce(|acc, e| {
                        parse_quote! {
                            #acc else #e
                        }
                    });

                let end_statement: Vec<Stmt> = parse_quote!(
                    #(#end_check)*

                    break;
                );

                let if_statements = if let Some(if_statements) = if_statements {
                    parse_quote! {
                        #if_statements else {
                            #(#end_statement)*
                        }
                    }
                } else {
                    end_statement
                };

                Ok(parse_quote! {
                    loop {
                        #(#ignore_whitespace_expression)*
                        #(#if_statements)*
                    }
                })
            }
            ElementOrder::None => {
                let field_visits = builder_element_field_visitor(
                    seq_access,
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

                let skip_unknown: Vec<Stmt> = match allow_unknown_children {
                    AllowUnknown::Any => {
                        // Currently, allow any unknown is not supported with strict ordering.
                        if matches!(order, ElementOrder::Strict) {
                            return Err(DeriveError::custom(
                                "An unknown element in any position is not allowed in strict order",
                            ));
                        }

                        let skip_ident = Ident::new("__skip", seq_access.span());
                        parse_quote! {
                            let #skip_ident = ::core::result::Result::unwrap_or(
                                ::xmlity::de::SeqAccess::next_element::<#ignored_any_ty>(#seq_access),
                                None
                            );

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
                        let skip_ident = Ident::new("__skip", seq_access.span());
                        parse_quote! {
                            let #skip_ident = ::core::result::Result::unwrap_or(
                                ::xmlity::de::SeqAccess::next_element::<#ignored_any_ty>(#seq_access),
                                None
                            );

                            if ::core::option::Option::is_none(&#skip_ident) {
                                break;
                            }

                            return Err(::xmlity::de::Error::unknown_child());
                        }
                    }
                };

                Ok(parse_quote! {
                    loop {
                        #(#ignore_whitespace_expression)*
                        #(#field_visits)*
                        #(#skip_unknown)*
                    }
                })
            }
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
                        FieldValueGroupOpts::Value(opts) => match self.order {
                            ElementOrder::Strict => {
                                parse_quote!(
                                    ::core::option::Option::expect(
                                        #builder_field_ident,
                                        "Should have been set by the time we get here. This is a bug in xmlity.",
                                    )
                                )
                            },
                            ElementOrder::None => {
                                deserialize_option_value_expr(
                                    &field_type,
                                    &parse_quote!(#builder_field_ident),
                                    opts.default_or_else(),
                                    self.order == ElementOrder::None && matches!(opts, ChildOpts::Value(_)),
                                    visitor_lifetime,
                                    error_type,
                                    &field_ident.to_string(),
                                )
                            },
                        } ,
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
