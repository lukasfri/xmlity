use std::borrow::Cow;

use proc_macro2::Span;
use syn::{
    parse_quote, DeriveInput, Expr, Field, Ident, ItemStruct, Lifetime, LifetimeParam, Stmt, Type,
    Variant,
};

use crate::{
    de::{
        common::{
            DeserializeBuilder, DeserializeBuilderExt, SeqVisitLoop, VisitorBuilder,
            VisitorBuilderExt,
        },
        constructor_expr, StructType,
    },
    options::{
        enums::{self, variants::ValueOpts},
        structs::{
            self,
            fields::{FieldOpts, FieldValueGroupOpts},
        },
        ElementOrder,
    },
    DeriveError, DeriveResult, FieldIdent, FieldWithOpts,
};

use super::values::StringLiteralDeserializeBuilder;

pub struct SerializeNoneStructBuilder<'a> {
    pub ast: &'a DeriveInput,
}

impl<'a> SerializeNoneStructBuilder<'a> {
    pub fn new(ast: &'a DeriveInput) -> Self {
        Self { ast }
    }

    pub fn field_decl(
        element_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, structs::fields::ChildOpts>>,
        group_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, structs::fields::GroupOpts>>,
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
        ident: &Ident,
        visitor_lifetime: &syn::Lifetime,
        access_type: &Type,
        element_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, structs::fields::ChildOpts>>,
        group_fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, structs::fields::GroupOpts>>,
        constructor_type: StructType,
    ) -> proc_macro2::TokenStream {
        let local_value_expressions_constructors =
            element_fields.into_iter()
            .map::<(_, Expr), _>(|FieldWithOpts {  field_ident, options, .. }| {
                let builder_field_ident = field_ident.to_named_ident();
                let expression = if options.should_unwrap_default() {
                    parse_quote! {
                        ::core::option::Option::unwrap_or_default(#builder_field_ident)
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

        constructor_expr(ident, value_expressions_constructors, &constructor_type)
    }

    pub fn seq_access(
        access_ident: &Ident,
        fields: impl IntoIterator<Item = FieldWithOpts<FieldIdent, FieldValueGroupOpts>> + Clone,
        allow_unknown_children: bool,
        order: ElementOrder,
    ) -> DeriveResult<Vec<Stmt>> {
        let visit = SeqVisitLoop::new(access_ident, allow_unknown_children, order, fields);

        let field_storage = visit.field_storage();
        let access_loop = visit.access_loop()?;

        Ok(parse_quote! {
            #field_storage

            #(#access_loop)*
        })
    }
}

impl VisitorBuilder for SerializeNoneStructBuilder<'_> {
    fn visit_seq_fn_body(
        &self,
        visitor_lifetime: &Lifetime,
        access_ident: &Ident,
        access_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let DeriveInput { data, .. } = &self.ast;

        let data_struct = match data {
            syn::Data::Struct(data_struct) => data_struct,
            _ => unreachable!(),
        };

        let fields = match &data_struct.fields {
            fields @ syn::Fields::Named(_) | fields @ syn::Fields::Unnamed(_) => fields,
            syn::Fields::Unit => unreachable!(),
        };

        let constructor_type = match fields {
            syn::Fields::Named(_) => StructType::Named,
            syn::Fields::Unnamed(_) => StructType::Unnamed,
            _ => unreachable!(),
        };

        let fields = match fields {
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

        let children_loop = if element_group_fields.clone().next().is_some() {
            Self::seq_access(
                access_ident,
                element_group_fields,
                false,
                ElementOrder::Loose,
            )?
        } else {
            Vec::new()
        };

        let constructor = Self::constructor_expr(
            &self.ast.ident,
            visitor_lifetime,
            access_type,
            element_fields.clone(),
            group_fields.clone(),
            constructor_type,
        );

        Ok(Some(parse_quote! {
            #(#getter_declarations)*

            #(#children_loop)*

            ::core::result::Result::Ok(#constructor)
        }))
    }

    fn visitor_definition(&self) -> Result<ItemStruct, DeriveError> {
        let DeriveInput {
            ident, generics, ..
        } = &self.ast;
        let non_bound_generics = crate::non_bound_generics(generics);

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

impl DeserializeBuilder for SerializeNoneStructBuilder<'_> {
    fn deserialize_fn_body(
        &self,
        deserializer_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let formatter_expecting = format!("struct {}", self.ast.ident);

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

pub struct EnumVisitorBuilder<'a> {
    ast: &'a DeriveInput,
    value_opts: Option<&'a enums::roots::RootValueOpts>,
}

impl<'a> EnumVisitorBuilder<'a> {
    pub fn new(ast: &'a DeriveInput) -> Self {
        Self {
            ast,
            value_opts: None,
        }
    }

    pub fn new_with_value_opts(
        ast: &'a DeriveInput,
        value_opts: &'a enums::roots::RootValueOpts,
    ) -> Self {
        Self {
            ast,
            value_opts: Some(value_opts),
        }
    }
}

impl VisitorBuilder for EnumVisitorBuilder<'_> {
    fn visit_seq_fn_body(
        &self,
        _visitor_lifetime: &Lifetime,
        access_ident: &Ident,
        _access_type: &Type,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let DeriveInput { ident, data, .. } = &self.ast;
        let data_enum = match &data {
            syn::Data::Enum(data_enum) => data_enum,
            _ => panic!("Wrong options. Only enums can be used for xelement."),
        };
        let variants = data_enum.variants.iter().collect::<Vec<_>>();

        let variants = variants.clone().into_iter().map::<Result<Expr, DeriveError>, _>(|variant @ Variant {
            ident: variant_ident,
            fields: variant_fields,
            ..
        }| {
            match variant_fields {
                syn::Fields::Named(_fields) => {
                    Err(DeriveError::Custom { message: "Deriving for named fields is not supported yet".to_string(), span: Span::call_site() })
                }
                syn::Fields::Unit  => {
                    let opts = enums::variants::VariantOpts::from_variant(variant)?;

                    let value = opts.as_ref().and_then(|a| match a {
                        enums::variants::VariantOpts::Value(ValueOpts {value: Some(value), ..}) => Some(value.clone()),
                        _ => None,
                    }).unwrap_or_else(|| {
                        self.value_opts
                            .as_ref()
                            .map(|a| a.rename_all)
                            .unwrap_or_default()
                            .apply_to_variant(&variant_ident.to_string())
                    });

                    let deserialize_test_ident = Ident::new("__DeserializeTest", Span::call_site());

                    let literal_deserializer = StringLiteralDeserializeBuilder::new(deserialize_test_ident.clone(), &value);
                    let definition = literal_deserializer.definition()?;
                    let deserialize_trait_impl = literal_deserializer.deserialize_trait_impl()?;
                    Ok(parse_quote! {
                        {
                            #definition
                            #deserialize_trait_impl
                            if let ::core::result::Result::Ok(::core::option::Option::Some(_v)) = ::xmlity::de::SeqAccess::next_element::<#deserialize_test_ident>(&mut #access_ident) {
                                return ::core::result::Result::Ok(#ident::#variant_ident);
                            }
                        }
                    })
                }
                syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1  => {
                    let Field {
                        ty: field_type,
                        ..
                    } = fields.unnamed.first().expect("This is guaranteed by the check above");

                    Ok(parse_quote! {
                        if let ::core::result::Result::Ok(::core::option::Option::Some(_v)) = ::xmlity::de::SeqAccess::next_element::<#field_type>(&mut #access_ident) {
                            return ::core::result::Result::Ok(#ident::#variant_ident(_v));
                        }
                    })
                }
                syn::Fields::Unnamed(_) => {
                    Err(DeriveError::Custom { message: "Cannot deserialize unnamed variants with more than one field".to_string(), span: Span::call_site() })
                }
            }
        }).collect::<Result<Vec<_>, _>>()?;
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
        let non_bound_generics = crate::non_bound_generics(generics);

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
