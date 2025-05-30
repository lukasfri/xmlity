use proc_macro2::Span;
use syn::{parse_quote, Expr, Ident, Item, Stmt};

use crate::{
    common::FieldIdent,
    options::{
        records::fields::{
            AttributeOpts, ChildOpts, FieldAttributeGroupOpts, FieldOpts, FieldValueGroupOpts,
        },
        FieldWithOpts, WithExpandedNameExt,
    },
    DeriveError, DeriveResult,
};
use quote::{quote, ToTokens};

use super::{
    builders::{SerializeAttributeBuilderExt, SerializeBuilderExt},
    serialize::SingleChildSerializeElementBuilder,
    serialize_attribute::SimpleSerializeAttributeBuilder,
};

pub fn attribute_field_serializer(
    access_ident: impl ToTokens,
    field_ident: &FieldIdent,
    item_type: &syn::Type,
    field_ident_to_expr: impl Fn(&FieldIdent) -> syn::Expr,
    opts: AttributeOpts,
) -> DeriveResult<proc_macro2::TokenStream> {
    let (items, serialize_expr) = match opts {
        AttributeOpts::Declared(opts) => {
            let wrapper_ident = Ident::new("__W", Span::call_site());

            let wrapper = SimpleSerializeAttributeBuilder {
                ident: &wrapper_ident,
                generics: &syn::Generics::default(),
                expanded_name: opts
                    .expanded_name(&field_ident.to_named_ident().to_string())
                    .into_owned(),
                preferred_prefix: opts.preferred_prefix,
                enforce_prefix: opts.enforce_prefix,
                item_type,
            };

            let definition = wrapper.struct_definition();
            let trait_impl = wrapper.serialize_attribute_trait_impl()?;
            let value_expr = wrapper.value_expression(&field_ident_to_expr(field_ident));
            let serialize_expr: Expr = parse_quote!(&#value_expr);

            (
                vec![Item::Struct(definition), Item::Impl(trait_impl)],
                serialize_expr,
            )
        }
        AttributeOpts::Deferred(_) => {
            let ser_value = field_ident_to_expr(field_ident);

            (vec![], ser_value)
        }
    };

    Ok(quote! {{
        #(#items)*
        ::xmlity::ser::SerializeAttributes::serialize_attribute(#access_ident, #serialize_expr)?;
    }})
}

pub fn attribute_group_fields_serializer(
    access_ident: impl ToTokens,
    fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, FieldAttributeGroupOpts>>,
    field_ident_to_expr: impl Fn(&FieldIdent) -> syn::Expr,
) -> DeriveResult<proc_macro2::TokenStream> {
    let fields = fields
    .into_iter()
    .map(|var_field| {
        let field_ident = &var_field.field_ident;

        match var_field.options {
            FieldAttributeGroupOpts::Attribute(opts) => {
                attribute_field_serializer(
                    &access_ident,
                    field_ident,
                    &var_field.field_type,
                    &field_ident_to_expr,
                    opts,
                )
            },
            FieldAttributeGroupOpts::Group(_) => {
                let ser_value = field_ident_to_expr(field_ident);

                Ok(quote! {
                    ::xmlity::ser::SerializationGroup::serialize_attributes(#ser_value, #access_ident)?;
                })
            },
        }
    }).collect::<DeriveResult<Vec<_>>>()?;

    Ok(quote! {
        #(#fields)*
    })
}

pub fn element_field_serializer(
    access_ident: impl ToTokens,
    field_ident: &FieldIdent,
    item_type: &syn::Type,
    field_ident_to_expr: impl Fn(&FieldIdent) -> syn::Expr,
    opts: ChildOpts,
) -> DeriveResult<proc_macro2::TokenStream> {
    let (prefix, serialize_expr, skip_serializing_if_expr): (Vec<_>, _, _) = match opts {
        ChildOpts::Value(value_opts) => {
            let value_expr = field_ident_to_expr(field_ident);

            let skip_serializing_if_expr: Option<syn::Expr> =
                value_opts.skip_serializing_if.map(|skip_serializing_if| {
                    parse_quote!(
                        #skip_serializing_if(&#value_expr)
                    )
                });

            (Vec::new(), value_expr, skip_serializing_if_expr)
        }
        ChildOpts::Element(opts) => {
            let wrapper_ident = Ident::new("__W", Span::call_site());

            let wrapper = SingleChildSerializeElementBuilder {
                ident: &wrapper_ident,
                expanded_name: opts
                    .expanded_name(field_ident.to_named_ident().to_string().as_str())
                    .into_owned(),
                preferred_prefix: opts.preferred_prefix,
                enforce_prefix: opts.enforce_prefix,
                item_type,
                group: opts.group,
                skip_serializing_if: None,
            };

            let definition = wrapper.struct_definition();
            let trait_impl = wrapper.serialize_trait_impl()?;
            let value_expr = field_ident_to_expr(field_ident);

            let serialize_expr = wrapper.value_expression(&value_expr);

            let skip_serializing_if_expr: Option<syn::Expr> =
                opts.skip_serializing_if.map(|skip_serializing_if| {
                    parse_quote!(
                        #skip_serializing_if(&#value_expr)
                    )
                });

            let wrapped_value_expr = parse_quote! {&#serialize_expr};

            (
                vec![Item::Struct(definition), Item::Impl(trait_impl)],
                wrapped_value_expr,
                skip_serializing_if_expr,
            )
        }
    };

    let serialize_element_stmt: Stmt = parse_quote!(
        ::xmlity::ser::SerializeSeq::serialize_element(#access_ident, #serialize_expr)?;
    );

    let serialize_element_expr = if let Some(skip_serializing_if_expr) = skip_serializing_if_expr {
        parse_quote! {
            if !#skip_serializing_if_expr {
                #serialize_element_stmt
            }
        }
    } else {
        serialize_element_stmt
    };

    Ok(quote! {
        {
            #(#prefix)*
            #serialize_element_expr
        }
    })
}

pub fn element_group_fields_serializer(
    access_ident: impl ToTokens,
    fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, FieldValueGroupOpts>>,
    field_ident_to_expr: impl Fn(&FieldIdent) -> syn::Expr,
) -> DeriveResult<proc_macro2::TokenStream> {
    let fields = fields
    .into_iter()
    .map::<DeriveResult<_>, _>(|var_field| {
        let field_ident = &var_field.field_ident;

        match var_field.options {
            FieldValueGroupOpts::Value(opts) => {
                element_field_serializer(&access_ident, field_ident, &var_field.field_type, |a| field_ident_to_expr(a), opts)
            },
            FieldValueGroupOpts::Group(_) => {
                let ser_value = field_ident_to_expr(field_ident);

                Ok(quote! {
                    ::xmlity::ser::SerializationGroup::serialize_children(#ser_value, #access_ident)?;
                })
            },
        }
    }).collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        #(#fields)*
    })
}

pub fn element_fields_serializer(
    access_ident: impl ToTokens,
    fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, ChildOpts>>,
    field_ident_to_expr: impl Fn(&FieldIdent) -> syn::Expr,
) -> DeriveResult<proc_macro2::TokenStream> {
    let fields = fields
        .into_iter()
        .map(|var_field| {
            element_field_serializer(
                &access_ident,
                &var_field.field_ident,
                &var_field.field_type,
                |a| field_ident_to_expr(a),
                var_field.options,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        #(#fields)*
    })
}

pub fn fields(
    ast: &syn::DeriveInput,
) -> Result<Vec<FieldWithOpts<FieldIdent, FieldOpts>>, DeriveError> {
    let syn::Data::Struct(syn::DataStruct { fields, .. }) = &ast.data else {
        unreachable!()
    };

    match fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .map(|f| {
                Ok(FieldWithOpts {
                    field_ident: FieldIdent::Named(f.ident.clone().expect("Named struct")),
                    options: FieldOpts::from_field(f)?,
                    field_type: f.ty.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>(),
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| {
                Ok(FieldWithOpts {
                    field_ident: FieldIdent::Indexed(syn::Index::from(i)),
                    options: FieldOpts::from_field(f)?,
                    field_type: f.ty.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>(),
        syn::Fields::Unit => Ok(vec![]),
    }
}

pub fn attribute_group_fields(
    fields: Vec<FieldWithOpts<FieldIdent, FieldOpts>>,
) -> Result<Vec<FieldWithOpts<FieldIdent, FieldAttributeGroupOpts>>, DeriveError> {
    Ok(fields
        .into_iter()
        .filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                FieldOpts::Attribute(opts) => Some(FieldAttributeGroupOpts::Attribute(opts)),
                FieldOpts::Group(opts) => Some(FieldAttributeGroupOpts::Group(opts)),
                FieldOpts::Value(_) => None,
            })
        })
        .collect())
}

pub fn element_group_fields(
    fields: Vec<FieldWithOpts<FieldIdent, FieldOpts>>,
) -> Result<Vec<FieldWithOpts<FieldIdent, FieldValueGroupOpts>>, DeriveError> {
    Ok(fields
        .into_iter()
        .filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                FieldOpts::Value(opts) => Some(FieldValueGroupOpts::Value(opts)),
                FieldOpts::Group(opts) => Some(FieldValueGroupOpts::Group(opts)),
                FieldOpts::Attribute(_) => None,
            })
        })
        .collect())
}

pub fn element_fields(
    fields: Vec<FieldWithOpts<FieldIdent, FieldOpts>>,
) -> Result<Vec<FieldWithOpts<FieldIdent, ChildOpts>>, DeriveError> {
    Ok(fields
        .into_iter()
        .filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                FieldOpts::Value(opts) => Some(opts),
                FieldOpts::Group(_) | FieldOpts::Attribute(_) => None,
            })
        })
        .collect())
}
