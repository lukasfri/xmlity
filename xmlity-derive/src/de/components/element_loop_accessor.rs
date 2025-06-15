use quote::format_ident;
use syn::{parse_quote, Expr, Stmt};

use crate::{
    common::FieldIdent,
    de::{
        common::{builder_attribute_field_visitor, deserialize_option_value_expr},
        components::SeqLoopAccessor,
    },
    derive::DeriveResult,
    options::{
        records::fields::{AttributeOpts, FieldOpts, FieldValueGroupOpts},
        AllowUnknown, ElementOrder, FieldWithOpts, IgnoreWhitespace,
    },
};

pub struct ElementLoopAccessor {
    children_loop_accessor: SeqLoopAccessor,
    allow_unknown_attributes: AllowUnknown,
    attribute_order: ElementOrder,
}

impl ElementLoopAccessor {
    pub fn new(
        allow_unknown_children: AllowUnknown,

        allow_unknown_attributes: AllowUnknown,
        children_order: ElementOrder,
        attribute_order: ElementOrder,
        ignore_whitespace: IgnoreWhitespace,
    ) -> Self {
        Self {
            children_loop_accessor: SeqLoopAccessor::new(
                allow_unknown_children,
                children_order,
                ignore_whitespace,
            ),
            allow_unknown_attributes,
            attribute_order,
        }
    }

    fn split_fields<F: IntoIterator<Item = FieldWithOpts<FieldIdent, FieldOpts>>>(
        fields: F,
    ) -> (
        impl Iterator<Item = FieldWithOpts<FieldIdent, AttributeOpts>>,
        impl Iterator<Item = FieldWithOpts<FieldIdent, FieldValueGroupOpts>>,
    ) {
        let (attribute_fields, child_group_fields) = fields
            .into_iter()
            .map(|a| match a.options {
                FieldOpts::Value(child_opts) => (
                    None,
                    Some(FieldWithOpts {
                        field_ident: a.field_ident,
                        field_type: a.field_type,
                        options: FieldValueGroupOpts::Value(child_opts),
                    }),
                ),
                FieldOpts::Attribute(attribute_opts) => (
                    Some(FieldWithOpts {
                        field_ident: a.field_ident,
                        field_type: a.field_type,
                        options: attribute_opts,
                    }),
                    None,
                ),
                FieldOpts::Group(group_opts) => (
                    None,
                    Some(FieldWithOpts {
                        field_ident: a.field_ident,
                        field_type: a.field_type,
                        options: FieldValueGroupOpts::Group(group_opts),
                    }),
                ),
            })
            .unzip::<_, _, Vec<_>, Vec<_>>();

        (
            attribute_fields.into_iter().flatten(),
            child_group_fields.into_iter().flatten(),
        )
    }

    pub fn field_definitions<F: IntoIterator<Item = FieldWithOpts<FieldIdent, FieldOpts>>>(
        &self,
        fields: F,
    ) -> DeriveResult<Vec<Stmt>> {
        let (attribute_fields, child_group_fields) = Self::split_fields(fields);

        let attributes_definitions: DeriveResult<Vec<Stmt>> = attribute_fields
            .map(
                |FieldWithOpts {
                     field_ident,
                     field_type,
                     ..
                 }| {
                    let builder_initializer: syn::Expr = parse_quote! {
                         ::core::option::Option::<#field_type>::None
                    };

                    let builder_field_ident = field_ident.to_named_ident();

                    Ok(parse_quote! {
                        let mut #builder_field_ident = #builder_initializer;
                    })
                },
            )
            .collect();

        let child_group_definitions = self
            .children_loop_accessor
            .field_definitions(child_group_fields)?;

        Ok(attributes_definitions?
            .into_iter()
            .chain(child_group_definitions)
            .collect())
    }

    pub fn attribute_access_loop<F: IntoIterator<Item = FieldWithOpts<FieldIdent, FieldOpts>>>(
        &self,
        fields: F,
        access_expr: &Expr,
    ) -> DeriveResult<Vec<Stmt>> {
        let Self {
            allow_unknown_attributes,
            attribute_order: attributes_order,
            ..
        } = self;

        let attribute_group_fields = fields
            .into_iter()
            .map(|a| a.map_options_opt(|a| a.attribute_group()))
            .flatten();

        let field_visits = builder_attribute_field_visitor(
            access_expr,
            |field| {
                let ident = field.to_named_ident();
                parse_quote! {#ident}
            },
            attribute_group_fields,
            parse_quote! {break;},
            match *attributes_order {
                ElementOrder::Strict => parse_quote! {break;},
                ElementOrder::None => parse_quote! {continue;},
            },
            parse_quote! {continue;},
            parse_quote! {},
            false,
        )?;

        let skip_unknown: Vec<Stmt> = match allow_unknown_attributes {
            AllowUnknown::Any => {
                let skip_ident = format_ident!("__skip");
                parse_quote! {
                    let #skip_ident = ::xmlity::de::AttributesAccess::next_attribute::<::xmlity::types::utils::IgnoredAny>(#access_expr).unwrap_or(None);
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
                let skip_ident = format_ident!("__skip");
                parse_quote! {
                    let #skip_ident = ::xmlity::de::AttributesAccess::next_attribute::<::xmlity::types::utils::IgnoredAny>(#access_expr).unwrap_or(None);
                    if ::core::option::Option::is_none(&#skip_ident) {
                        break;
                    }

                    return Err(::xmlity::de::Error::unknown_child());
                }
            }
        };

        match *attributes_order {
            ElementOrder::Strict => field_visits
                .into_iter()
                .map(|field_visit| {
                    Ok(parse_quote! {
                        loop {
                            #field_visit
                            #(#skip_unknown)*
                        }
                    })
                })
                .collect(),
            ElementOrder::None => Ok(parse_quote! {
                loop {
                    #(#field_visits)*
                    #(#skip_unknown)*
                }
            }),
        }
    }

    pub fn children_access_loop<F: IntoIterator<Item = FieldWithOpts<FieldIdent, FieldOpts>>>(
        &self,
        fields: F,
        access_expr: &Expr,
    ) -> DeriveResult<Vec<Stmt>> {
        let child_group_fields = fields
            .into_iter()
            .map(|a| a.map_options_opt(|a| a.value_group()))
            .flatten();

        self.children_loop_accessor
            .access_loop(child_group_fields, access_expr)
    }

    pub fn value_expressions<F: IntoIterator<Item = FieldWithOpts<FieldIdent, FieldOpts>>>(
        &self,
        fields: F,
        visitor_lifetime: &syn::Lifetime,
        error_type: &syn::Type,
    ) -> DeriveResult<Vec<(FieldIdent, Expr)>> {
        let (attribute_fields, child_group_fields) = Self::split_fields(fields);

        let attribute_exprs = attribute_fields
            .map(
                |FieldWithOpts {
                     field_ident,
                     field_type,
                     options,
                 }| {
                    let builder_field_ident = field_ident.to_named_ident();

                    let expr: Expr = deserialize_option_value_expr(
                        &field_type,
                        &parse_quote!(#builder_field_ident),
                        options.default_or_else(),
                        false,
                        visitor_lifetime,
                        error_type,
                        &field_ident.to_string(),
                    );

                    (field_ident, expr)
                },
            )
            .collect::<Vec<_>>();

        let value_group_exprs = self.children_loop_accessor.value_expressions(
            child_group_fields,
            visitor_lifetime,
            error_type,
        )?;

        Ok(attribute_exprs
            .into_iter()
            .chain(value_group_exprs)
            .collect())
    }
}
