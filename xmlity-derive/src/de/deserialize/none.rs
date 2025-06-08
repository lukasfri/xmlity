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
        AllowUnknown, ElementOrder, FieldWithOpts, IgnoreWhitespace,
    },
    DeriveError,
};

use super::{parse_enum_variant_derive_input, variant::DeserializeVariantBuilder, RecordInput};

pub struct RecordDeserializeValueBuilder<'a, T: Fn(syn::Expr) -> syn::Expr> {
    pub input: &'a RecordInput<'a, T>,
    pub ignore_whitespace: IgnoreWhitespace,
    pub allow_unknown_children: AllowUnknown,
    pub children_order: ElementOrder,
    pub value: Option<String>,
}

impl<'a, T: Fn(syn::Expr) -> syn::Expr> RecordDeserializeValueBuilder<'a, T> {
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
        error_type: &syn::Type,
        element_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, records::fields::ChildOpts>>,
        group_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, records::fields::GroupOpts>>,
        constructor_type: StructType,
    ) -> syn::Expr {
        let local_value_expressions_constructors =
            element_fields.into_iter()
            .map::<(_, Expr), _>(|FieldWithOpts {  field_ident, options, field_type }| {
                let builder_field_ident = field_ident.to_named_ident();
                let expression = if let Some(default_or_else) = options.default_or_else() {
                    parse_quote! {
                        ::core::option::Option::unwrap_or_else(#builder_field_ident, #default_or_else)
                    }
                } else {
                    parse_quote! {
                        ::core::result::Result::map_err(
                            ::core::option::Option::map_or_else(
                                #builder_field_ident,
                                || <#field_type as ::xmlity::Deserialize<#visitor_lifetime>>::deserialize_seq(
                                    ::xmlity::types::utils::NoneDeserializer::<#error_type>::new(),
                                ),
                                |__v| ::core::result::Result::Ok(__v)
                            ),
                            |_|  ::xmlity::de::Error::missing_field(stringify!(#field_ident))
                        )?
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
                    ::xmlity::de::DeserializationGroupBuilder::finish::<#error_type>(#builder_field_ident)?
                };

                (field_ident, expression)
            },
        );

        let value_expressions_constructors =
            local_value_expressions_constructors.chain(group_value_expressions_constructors);

        constructor_expr(path, value_expressions_constructors, &constructor_type)
    }

    fn str_value_body(
        &self,
        value: &str,
        value_ident: &Ident,
        visitor_lifetime: &Lifetime,
        _access_type: &Type,
        error_type: &Type,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let constructor = (self.input.wrapper_function)(Self::constructor_expr(
            self.input.constructor_path.as_ref(),
            visitor_lifetime,
            error_type,
            [],
            [],
            StructType::Unit,
        ));

        Ok(parse_quote! {
            if ::core::primitive::str::trim(::core::ops::Deref::deref(&#value_ident)) == #value {
                return ::core::result::Result::Ok(#constructor);
            }

            ::core::result::Result::Err(::xmlity::de::Error::no_possible_variant("Value"))
        })
    }

    fn should_deserialize_as_str(&self) -> Option<&str> {
        if matches!(self.input.fields, StructTypeWithFields::Unit) && self.value.is_some() {
            self.value.as_deref()
        } else {
            None
        }
    }

    // This is an option because sometimes we know that a type cannot be constructed from an empty, for example when it has an inline element without a default value.
    pub fn empty_constructor_expr(
        path: &syn::Path,
        visitor_lifetime: &Lifetime,
        error_type: &Type,
        fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, FieldValueGroupOpts>>,
        constructor_type: StructType,
    ) -> Option<syn::Expr> {
        let value_expressions_constructors = fields.into_iter().map::<Option<(_, Expr)>, _>(
            |FieldWithOpts {
                 field_ident,
                 options,
                 field_type,
                 ..
             }| {
                let expression = match options {
                    FieldValueGroupOpts::Value(options) => {
                        if let Some(default_or_else) = options.default_or_else() {
                            Some(parse_quote! {
                                (#default_or_else)()
                            })
                        } else {
                            match options {
                                records::fields::ChildOpts::Value(_) =>  {
                                    Some(parse_quote! {
                                        ::core::result::Result::map_err(
                                            <#field_type as ::xmlity::Deserialize<#visitor_lifetime>>::deserialize_seq(
                                                ::xmlity::types::utils::NoneDeserializer::<#error_type>::new(),
                                            ),
                                            |_| <#error_type as ::xmlity::de::Error>::missing_field(stringify!(#field_ident)),
                                        )?
                                    })
                                }
                                ,
                                records::fields::ChildOpts::Element(_) => None,
                            }
                        }
                    },
                    FieldValueGroupOpts::Group(_options) => {
                        Some(parse_quote! {
                            <<#field_type as ::xmlity::DeserializationGroup>::Builder as ::xmlity::de::DeserializationGroupBuilder>::finish::<#error_type>(
                                <#field_type as ::xmlity::DeserializationGroup>::builder()
                            )?
                        })
                    },
                };

                expression.map(|expression| (field_ident, expression))
            },
        ).collect::<Option<Vec<_>>>()?;

        Some(constructor_expr(
            path,
            value_expressions_constructors,
            &constructor_type,
        ))
    }
}

impl<T: Fn(syn::Expr) -> syn::Expr> VisitorBuilder for RecordDeserializeValueBuilder<'_, T> {
    fn visit_text_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        access_ident: &Ident,
        access_type: &Type,
        error_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let Some(value) = self.should_deserialize_as_str() else {
            return Ok(None);
        };

        let str_ident = Ident::new("__value_str", Span::mixed_site());

        let str_body =
            self.str_value_body(value, &str_ident, visitor_lifetime, access_type, error_type)?;

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
        error_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let Some(value) = self.should_deserialize_as_str() else {
            return Ok(None);
        };

        let str_ident = Ident::new("__value_str", Span::mixed_site());

        let str_body =
            self.str_value_body(value, &str_ident, visitor_lifetime, access_type, error_type)?;

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

        let children_loop = element_group_fields
            .clone()
            .next()
            .is_some()
            .then(|| {
                SeqVisitLoop::new(
                    access_ident,
                    self.allow_unknown_children,
                    self.children_order,
                    self.ignore_whitespace,
                    element_group_fields,
                )
                .access_loop()
            })
            .transpose()?
            .unwrap_or_default();

        let error_type: syn::Type =
            parse_quote!(<#access_type as ::xmlity::de::SeqAccess<#visitor_lifetime>>::Error);

        let constructor = (self.input.wrapper_function)(Self::constructor_expr(
            self.input.constructor_path.as_ref(),
            visitor_lifetime,
            &error_type,
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

    fn visit_none_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        error_type: &Type,
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

        let fields = fields.clone().into_iter().filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                FieldOpts::Value(opts) => Some(FieldValueGroupOpts::Value(opts)),
                FieldOpts::Group(opts) => Some(FieldValueGroupOpts::Group(opts)),
                _ => None,
            })
        });

        let Some(constructor) = Self::empty_constructor_expr(
            self.input.constructor_path.as_ref(),
            visitor_lifetime,
            error_type,
            fields,
            constructor_type,
        ) else {
            return Ok(None);
        };

        let constructor = (self.input.wrapper_function)(constructor);

        Ok(Some(parse_quote! {
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
            && self.value.is_some()
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

    pub fn variant_deserialize_definition(
        &self,
        variant: &syn::Variant,
    ) -> Result<Vec<syn::Item>, DeriveError> {
        let DeriveInput {
            ident,
            generics,
            data: syn::Data::Enum(data),
            ..
        } = &self.ast
        else {
            unreachable!("Should already have been checked.")
        };

        let fallible_enum = data.variants.len() > 1;

        let mut variant_opts = records::roots::DeserializeRootOpts::parse(&variant.attrs)?;
        if let DeserializeRootOpts::Value(records::roots::RootValueOpts {
            value: value @ None,
            ..
        }) = &mut variant_opts
        {
            let ident_value = self
                .value_opts
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

        let record = parse_enum_variant_derive_input(ident, generics, variant, fallible_enum)?;

        let builder = DeserializeVariantBuilder::new(&record, &variant_opts);

        let definition = builder.definition();
        let deserialize_trait_impl = builder.deserialize_trait_impl()?;

        Ok(vec![definition.into(), deserialize_trait_impl.into()])
    }

    pub fn variant_deserialize_expr(
        &self,
        variant: &syn::Variant,
        access_ident: &Ident,
    ) -> Result<Expr, DeriveError> {
        let DeriveInput {
            ident,
            generics,
            data: syn::Data::Enum(data),
            ..
        } = &self.ast
        else {
            unreachable!("Should already have been checked.")
        };

        let fallible_enum = data.variants.len() > 1;

        let mut variant_opts = records::roots::DeserializeRootOpts::parse(&variant.attrs)?;
        if let DeserializeRootOpts::Value(records::roots::RootValueOpts {
            value: value @ None,
            ..
        }) = &mut variant_opts
        {
            let ident_value = self
                .value_opts
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

        let record = parse_enum_variant_derive_input(ident, generics, variant, fallible_enum)?;

        let builder = DeserializeVariantBuilder::new(&record, &variant_opts);
        let inner_access = builder.value_access_ident();
        let non_bound_generics = non_bound_generics(builder.record.generics.as_ref());

        let variant_deserializer_ident = builder.record.impl_for_ident.as_ref();
        let variant_deserializer_type: syn::Type =
            parse_quote!( #variant_deserializer_ident #non_bound_generics);

        Ok(parse_quote! {
            {
                if let ::core::result::Result::Ok(::core::option::Option::Some(__v)) = ::xmlity::de::SeqAccess::next_element::<#variant_deserializer_type>(&mut #access_ident) {
                    return ::core::result::Result::Ok(__v.#inner_access);
                }
            }
        })
    }

    pub fn variant_deserialize_none_expr(
        &self,
        variant: &syn::Variant,
        visitor_lifetime: &Lifetime,
        error_type: &Type,
    ) -> Result<syn::Expr, DeriveError> {
        let DeriveInput {
            ident,
            generics,
            data: syn::Data::Enum(data),
            ..
        } = &self.ast
        else {
            unreachable!("Should already have been checked.")
        };

        let fallible_enum = data.variants.len() > 1;

        let mut variant_opts = records::roots::DeserializeRootOpts::parse(&variant.attrs)?;
        if let DeserializeRootOpts::Value(records::roots::RootValueOpts {
            value: value @ None,
            ..
        }) = &mut variant_opts
        {
            let ident_value = self
                .value_opts
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

        let record = parse_enum_variant_derive_input(ident, generics, variant, fallible_enum)?;

        let builder = DeserializeVariantBuilder::new(&record, &variant_opts);
        let inner_access = builder.value_access_ident();
        let non_bound_generics = non_bound_generics(builder.record.generics.as_ref());

        let variant_deserializer_ident = builder.record.impl_for_ident.as_ref();
        let variant_deserializer_type: syn::Type =
            parse_quote!( #variant_deserializer_ident #non_bound_generics);

        Ok(parse_quote! {
            {
                if let ::core::result::Result::Ok(__v) = <#variant_deserializer_type as ::xmlity::Deserialize<#visitor_lifetime>>::deserialize_seq(
                        ::xmlity::types::utils::NoneDeserializer::<#error_type>::new(),
                    ) {
                    return ::core::result::Result::Ok(__v.#inner_access);
                }
            }
        })
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
            data: syn::Data::Enum(data),
            ..
        } = &self.ast
        else {
            unreachable!("Should already have been checked.")
        };

        let variants = data
            .variants
            .iter()
            .map::<Result<Expr, DeriveError>, _>(|variant| {
                self.variant_deserialize_expr(variant, access_ident)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let ident_string = ident.to_string();

        Ok(Some(parse_quote! {
            #(#variants)*

            ::core::result::Result::Err(::xmlity::de::Error::no_possible_variant(#ident_string))
        }))
    }

    // TODO: This function really needs to be unified with `visit_seq_fn_body`, so that variant deserialize definitions are not duplicated.
    fn visit_none_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        error_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let DeriveInput {
            ident,
            data: syn::Data::Enum(data),
            ..
        } = &self.ast
        else {
            unreachable!("Should already have been checked.")
        };

        let variants = data
            .variants
            .iter()
            .map::<Result<Expr, DeriveError>, _>(|variant| {
                self.variant_deserialize_none_expr(variant, visitor_lifetime, error_type)
            })
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
        let DeriveInput {
            data: syn::Data::Enum(data),
            ..
        } = &self.ast
        else {
            unreachable!("Should already have been checked.")
        };

        let formatter_expecting = format!("enum {}", self.ast.ident);

        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());

        let visitor_def = <Self as VisitorBuilder>::visitor_definition(self)?;
        let visitor_trait_impl = <Self as VisitorBuilderExt>::visitor_trait_impl(
            self,
            &visitor_ident,
            &formatter_expecting,
        )?;

        let sub_serializer_defs = data
            .variants
            .iter()
            .map(|v| self.variant_deserialize_definition(v))
            .collect::<Result<Vec<_>, DeriveError>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Ok(parse_quote! {
            #(#sub_serializer_defs)*

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
