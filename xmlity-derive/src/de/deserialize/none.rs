use std::borrow::Cow;

use proc_macro2::Span;
use syn::{parse_quote, DeriveInput, Expr, Ident, ItemStruct, Lifetime, LifetimeParam, Stmt, Type};

use crate::{
    common::{constructor_expr, non_bound_generics, FieldIdent, StructType, StructTypeWithFields},
    de::{
        builders::{DeserializeBuilder, DeserializeBuilderExt, VisitorBuilder, VisitorBuilderExt},
        common::SeqVisitLoop,
    },
    options::{
        enums::{self},
        records::{
            self,
            fields::{FieldOpts, FieldValueGroupOpts},
            roots::DeserializeRootOpts,
        },
        ElementOrder, FieldWithOpts,
    },
    DeriveError, DeriveResult,
};

use super::{parse_enum_variant_derive_input, variant::DeserializeVariantBuilder, RecordInput};

pub struct RecordDeserializeValueBuilder<'a, T: Fn(syn::Expr) -> syn::Expr> {
    pub input: &'a RecordInput<'a, T>,
    pub options: Option<&'a records::roots::RootValueOpts>,
}

impl<'a, T: Fn(syn::Expr) -> syn::Expr> RecordDeserializeValueBuilder<'a, T> {
    pub fn new(
        input: &'a RecordInput<'a, T>,
        options: Option<&'a records::roots::RootValueOpts>,
    ) -> Self {
        Self { input, options }
    }

    pub fn field_decl(
        element_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, records::fields::ChildOpts>>,
        group_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, records::fields::GroupOpts>>,
    ) -> Vec<Stmt> {
        let getter_declarations = element_fields.into_iter().map::<Stmt, _>(
                |FieldWithOpts {
                     field_ident,
                     field_type,
                     ..
                 }| {
                    let builder_field_ident = field_ident.to_named_ident();
                    parse_quote! {
                        let mut #builder_field_ident = ::core::option::Option::<#field_type>::None;
                    }
                },
            ).chain(group_fields.into_iter().map::<Stmt, _>(
                |FieldWithOpts {
                     field_ident,
                     field_type,
                     ..
                 }| {
                    let builder_field_ident = field_ident.to_named_ident();
                    parse_quote! {
                        let mut #builder_field_ident = <#field_type as ::xmlity::de::DeserializationGroup>::builder();
                    }
                },
            ));

        parse_quote! {
            #(#getter_declarations)*
        }
    }

    pub fn constructor_expr(
        path: &syn::Path,
        visitor_lifetime: &syn::Lifetime,
        access_type: &Type,
        element_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, records::fields::ChildOpts>>,
        group_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, records::fields::GroupOpts>>,
        constructor_type: StructType,
    ) -> syn::Expr {
        let local_value_expressions_constructors =
            element_fields.into_iter()
            .map::<(_, Expr), _>(|FieldWithOpts {  field_ident, options, .. }| {
                let builder_field_ident = field_ident.to_named_ident();
                let expression = if let Some(default_or_else) = options.default_or_else() {
                    parse_quote! {
                        ::core::option::Option::unwrap_or_else(#builder_field_ident, #default_or_else)
                    }
                } else {
                    parse_quote! {
                        ::core::option::Option::ok_or(#builder_field_ident, ::xmlity::de::Error::missing_field(stringify!(#field_ident)))?
                    }
                };
                (field_ident, expression)
            });
        let group_value_expressions_constructors = group_fields.into_iter().map::<(_, Expr), _>(
            |FieldWithOpts {
                 field_ident,
                 ..
             }| {
                let builder_field_ident = field_ident.to_named_ident();
                let expression = parse_quote! {
                    ::xmlity::de::DeserializationGroupBuilder::finish::<<#access_type as ::xmlity::de::AttributesAccess<#visitor_lifetime>>::Error>(#builder_field_ident)?
                };

                (field_ident, expression)
            },
        );

        let value_expressions_constructors =
            local_value_expressions_constructors.chain(group_value_expressions_constructors);

        constructor_expr(path, value_expressions_constructors, &constructor_type)
    }

    pub fn seq_access(
        access_ident: &Ident,
        fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, FieldValueGroupOpts>> + Clone,
        allow_unknown_children: bool,
        order: ElementOrder,
        ignore_whitespace: bool,
    ) -> DeriveResult<Vec<Stmt>> {
        let visit = SeqVisitLoop::new(
            access_ident,
            allow_unknown_children,
            order,
            fields,
            ignore_whitespace,
        );

        let field_storage = visit.field_storage();
        let access_loop = visit.access_loop()?;

        Ok(parse_quote! {
            #field_storage

            #(#access_loop)*
        })
    }

    fn str_value_body(
        &self,
        value: &str,
        value_ident: &Ident,
        visitor_lifetime: &Lifetime,
        access_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let constructor = (self.input.wrapper_function)(Self::constructor_expr(
            self.input.constructor_path.as_ref(),
            visitor_lifetime,
            access_type,
            [],
            [],
            StructType::Unit,
        ));

        Ok(Some(parse_quote! {
            if ::core::primitive::str::trim(::core::ops::Deref::deref(&#value_ident)) == #value {
                return ::core::result::Result::Ok(#constructor);
            }

            ::core::result::Result::Err(::xmlity::de::Error::no_possible_variant("Value"))
        }))
    }

    fn should_deserialize_as_str(&self) -> Option<&str> {
        if matches!(self.input.fields, StructTypeWithFields::Unit)
            && self.options.is_some_and(|a| a.value.is_some())
        {
            self.options.as_ref().unwrap().value.as_deref()
        } else {
            None
        }
    }
}

impl<T: Fn(syn::Expr) -> syn::Expr> VisitorBuilder for RecordDeserializeValueBuilder<'_, T> {
    fn visit_text_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        access_ident: &Ident,
        access_type: &Type,
        _error_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let Some(value) = self.should_deserialize_as_str() else {
            return Ok(None);
        };

        let str_ident = Ident::new("__value_str", Span::mixed_site());

        let str_body = self.str_value_body(value, &str_ident, visitor_lifetime, access_type)?;

        let Some(str_body) = str_body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            let #str_ident = ::xmlity::de::XmlText::as_str(&#access_ident);
            #(#str_body)*
        }))
    }

    fn visit_cdata_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        access_ident: &Ident,
        access_type: &Type,
        _error_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let Some(value) = self.should_deserialize_as_str() else {
            return Ok(None);
        };

        let str_ident = Ident::new("__value_str", Span::mixed_site());

        let str_body = self.str_value_body(value, &str_ident, visitor_lifetime, access_type)?;

        let Some(str_body) = str_body else {
            return Ok(None);
        };

        Ok(Some(parse_quote! {
            let #str_ident = ::xmlity::de::XmlCData::as_str(&#access_ident);
            #(#str_body)*
        }))
    }

    fn visit_seq_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        access_ident: &Ident,
        access_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let RecordInput { fields, .. } = &self.input;

        // Only text match
        if self.should_deserialize_as_str().is_some() {
            return Ok(None);
        }

        let (constructor_type, fields) = match fields {
            StructTypeWithFields::Named(fields) => (
                StructType::Named,
                fields
                    .iter()
                    .cloned()
                    .map(|a| a.map_ident(FieldIdent::Named))
                    .collect(),
            ),
            StructTypeWithFields::Unnamed(fields) => (
                StructType::Unnamed,
                fields
                    .iter()
                    .cloned()
                    .map(|a| a.map_ident(FieldIdent::Indexed))
                    .collect(),
            ),
            StructTypeWithFields::Unit => (StructType::Unit, vec![]),
        };

        let element_fields = fields.clone().into_iter().filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                FieldOpts::Value(opts) => Some(opts),
                _ => None,
            })
        });

        let group_fields = fields.clone().into_iter().filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                FieldOpts::Group(opts) => Some(opts),
                _ => None,
            })
        });

        let getter_declarations = Self::field_decl(element_fields.clone(), group_fields.clone());

        let element_group_fields = fields.clone().into_iter().filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                FieldOpts::Value(opts) => Some(FieldValueGroupOpts::Value(opts)),
                FieldOpts::Group(opts) => Some(FieldValueGroupOpts::Group(opts)),
                FieldOpts::Attribute(_) => None,
            })
        });

        let ignore_whitespace = self
            .options
            .as_ref()
            .and_then(|a| a.ignore_whitespace)
            .unwrap_or(true);

        let children_loop = if element_group_fields.clone().next().is_some() {
            Self::seq_access(
                access_ident,
                element_group_fields,
                false,
                ElementOrder::Loose,
                ignore_whitespace,
            )?
        } else {
            Vec::new()
        };

        let constructor = (self.input.wrapper_function)(Self::constructor_expr(
            self.input.constructor_path.as_ref(),
            visitor_lifetime,
            access_type,
            element_fields.clone(),
            group_fields.clone(),
            constructor_type,
        ));

        Ok(Some(parse_quote! {
            #(#getter_declarations)*

            #(#children_loop)*

            ::core::result::Result::Ok(#constructor)
        }))
    }

    fn visitor_definition(&self) -> Result<ItemStruct, DeriveError> {
        let RecordInput {
            impl_for_ident: ident,
            generics,
            ..
        } = &self.input;
        let non_bound_generics = non_bound_generics(generics);

        let mut deserialize_generics = generics.as_ref().clone();

        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());
        let visitor_lifetime = Lifetime::new("'__visitor", Span::mixed_site());

        deserialize_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new(visitor_lifetime.clone())),
        );

        Ok(parse_quote! {
            struct #visitor_ident #deserialize_generics {
                marker: ::core::marker::PhantomData<#ident #non_bound_generics>,
                lifetime: ::core::marker::PhantomData<&#visitor_lifetime ()>,
            }
        })
    }

    fn visitor_ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(self.input.impl_for_ident.as_ref())
    }

    fn visitor_generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(self.input.generics.as_ref())
    }
}

impl<T: Fn(syn::Expr) -> syn::Expr> DeserializeBuilder for RecordDeserializeValueBuilder<'_, T> {
    fn deserialize_fn_body(
        &self,
        deserializer_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let formatter_expecting = format!("struct {}", self.input.impl_for_ident);

        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());

        let visitor_def = <Self as VisitorBuilder>::visitor_definition(self)?;
        let visitor_trait_impl = <Self as VisitorBuilderExt>::visitor_trait_impl(
            self,
            &visitor_ident,
            &formatter_expecting,
        )?;

        let deserialize_expr: syn::Expr = if matches!(self.input.fields, StructTypeWithFields::Unit)
            && self.options.is_some_and(|a| a.value.is_some())
        {
            parse_quote!(
                ::xmlity::de::Deserializer::deserialize_any(#deserializer_ident, #visitor_ident {
                    lifetime: ::core::marker::PhantomData,
                    marker: ::core::marker::PhantomData,
                })
            )
        } else {
            parse_quote!(
                ::xmlity::de::Deserializer::deserialize_seq(#deserializer_ident, #visitor_ident {
                    lifetime: ::core::marker::PhantomData,
                    marker: ::core::marker::PhantomData,
                })
            )
        };

        Ok(parse_quote! {
            #visitor_def

            #visitor_trait_impl

            #deserialize_expr
        })
    }

    fn ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(self.input.impl_for_ident.as_ref())
    }

    fn generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(self.input.generics.as_ref())
    }
}

pub struct EnumVisitorBuilder<'a> {
    ast: &'a DeriveInput,
    value_opts: Option<&'a enums::roots::RootValueOpts>,
}

impl<'a> EnumVisitorBuilder<'a> {
    pub fn new(ast: &'a DeriveInput, value_opts: Option<&'a enums::roots::RootValueOpts>) -> Self {
        Self { ast, value_opts }
    }
}

impl VisitorBuilder for EnumVisitorBuilder<'_> {
    fn visit_seq_fn_body(
        &self,
        _visitor_lifetime: &Lifetime,
        access_ident: &Ident,
        _access_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let DeriveInput {
            ident,
            generics,
            data,
            ..
        } = &self.ast;
        let data_enum = match &data {
            syn::Data::Enum(data_enum) => data_enum,
            _ => panic!("Wrong options. Only enums can be used for xelement."),
        };
        let variants = data_enum.variants.iter().collect::<Vec<_>>();

        let variants = variants
            .clone()
            .into_iter()
            .map::<Result<Expr, DeriveError>, _>(
                |variant | {
                    let mut variant_opts = records::roots::DeserializeRootOpts::parse(&variant.attrs)?;
                    if let DeserializeRootOpts::Value(records::roots::RootValueOpts {
                        value: value @ None,
                        ..
                    }) = &mut variant_opts {
                        let ident_value = self.value_opts
                            .as_ref()
                            .map(|a| a.rename_all)
                            .unwrap_or_default()
                            .apply_to_variant(&variant.ident.to_string());

                        *value = Some(ident_value);
                    }
                    if let DeserializeRootOpts::None = variant_opts {
                        let ident_value = self
                            .value_opts
                            .as_ref()
                            .map(|a| a.rename_all)
                            .unwrap_or_default()
                            .apply_to_variant(&variant.ident.to_string());

                        variant_opts = DeserializeRootOpts::Value(records::roots::RootValueOpts {
                            value: Some(ident_value),
                            ..Default::default()
                        });
                    }

                    let record =
                        parse_enum_variant_derive_input(ident, generics, variant, variants.len() > 1)?;

                    let builder = DeserializeVariantBuilder::new(&record, &variant_opts);
                    let inner_access = builder.value_access_ident();
                    let non_bound_generics = non_bound_generics(builder.record.generics.as_ref());

                    let variant_deserializer_ident = builder.record.impl_for_ident.as_ref();
                    let variant_deserializer_type: syn::Type = parse_quote!( #variant_deserializer_ident #non_bound_generics);

                    let definition = builder.definition();
                    let deserialize_trait_impl = builder.deserialize_trait_impl()?;
                    Ok(parse_quote! {
                        {
                            #definition
                            #deserialize_trait_impl
                            if let ::core::result::Result::Ok(::core::option::Option::Some(__v)) = ::xmlity::de::SeqAccess::next_element::<#variant_deserializer_type>(&mut #access_ident) {
                                return ::core::result::Result::Ok(__v.#inner_access);
                            }
                        }
                    })
                },
            )
            .collect::<Result<Vec<_>, _>>()?;
        let ident_string = ident.to_string();

        Ok(Some(parse_quote! {
            #(#variants)*

            ::core::result::Result::Err(::xmlity::de::Error::no_possible_variant(#ident_string))
        }))
    }

    fn visitor_definition(&self) -> Result<syn::ItemStruct, DeriveError> {
        let DeriveInput {
            ident, generics, ..
        } = &self.ast;
        let non_bound_generics = non_bound_generics(generics);

        let mut deserialize_generics = (*generics).to_owned();

        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());
        let visitor_lifetime = Lifetime::new("'__visitor", Span::mixed_site());

        deserialize_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(LifetimeParam::new(visitor_lifetime.clone())),
        );

        Ok(parse_quote! {
            struct #visitor_ident #deserialize_generics {
                marker: ::core::marker::PhantomData<#ident #non_bound_generics>,
                lifetime: ::core::marker::PhantomData<&#visitor_lifetime ()>,
            }
        })
    }

    fn visitor_ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(&self.ast.ident)
    }

    fn visitor_generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(&self.ast.generics)
    }
}

impl DeserializeBuilder for EnumVisitorBuilder<'_> {
    fn deserialize_fn_body(
        &self,
        deserializer_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let formatter_expecting = format!("enum {}", self.ast.ident);

        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());

        let visitor_def = <Self as VisitorBuilder>::visitor_definition(self)?;
        let visitor_trait_impl = <Self as VisitorBuilderExt>::visitor_trait_impl(
            self,
            &visitor_ident,
            &formatter_expecting,
        )?;

        Ok(parse_quote! {
            #visitor_def

            #visitor_trait_impl

            ::xmlity::de::Deserializer::deserialize_seq(#deserializer_ident, #visitor_ident {
                lifetime: ::core::marker::PhantomData,
                marker: ::core::marker::PhantomData,
            })
        })
    }

    fn ident(&self) -> Cow<'_, Ident> {
        Cow::Borrowed(&self.ast.ident)
    }

    fn generics(&self) -> Cow<'_, syn::Generics> {
        Cow::Borrowed(&self.ast.generics)
    }
}
