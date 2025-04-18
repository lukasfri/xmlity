use proc_macro2::Span;
use syn::{
    parse_quote, spanned::Spanned, DeriveInput, Expr, ExprIf, Field, Ident, ItemStruct, Lifetime,
    LifetimeParam, Stmt, Variant,
};

use crate::{
    de::{
        common::{DeserializeBuilder, SeqVisitLoop, VisitorBuilder, VisitorBuilderExt},
        constructor_expr, StructType,
    },
    options::{
        ElementOrder, XmlityFieldDeriveOpts, XmlityFieldGroupDeriveOpts,
        XmlityFieldValueDeriveOpts, XmlityFieldValueGroupDeriveOpts,
    },
    DeriveError, DeriveResult, DeserializeField, FieldIdent,
};

pub struct SerializeNoneStructBuilder;

impl SerializeNoneStructBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn field_decl(
        element_fields: impl IntoIterator<
            Item = DeserializeField<FieldIdent, XmlityFieldValueDeriveOpts>,
        >,
        group_fields: impl IntoIterator<Item = DeserializeField<FieldIdent, XmlityFieldGroupDeriveOpts>>,
    ) -> Vec<Stmt> {
        let getter_declarations = element_fields.into_iter().map::<Stmt, _>(
                |DeserializeField {
                     builder_field_ident,
                     field_type,
                     ..
                 }| {
                    parse_quote! {
                        let mut #builder_field_ident = ::core::option::Option::<#field_type>::None;
                    }
                },
            ).chain(group_fields.into_iter().map::<Stmt, _>(
                |DeserializeField {
                     builder_field_ident,
                     field_type,
                     ..
                 }| {
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
        element_fields: impl IntoIterator<
            Item = DeserializeField<FieldIdent, XmlityFieldValueDeriveOpts>,
        >,
        group_fields: impl IntoIterator<Item = DeserializeField<FieldIdent, XmlityFieldGroupDeriveOpts>>,
        constructor_type: StructType,
    ) -> proc_macro2::TokenStream {
        let local_value_expressions_constructors =
            element_fields.into_iter().map(|a| a.map_options(XmlityFieldValueGroupDeriveOpts::Value))
            .map::<(_, Expr), _>(|DeserializeField { builder_field_ident, field_ident, options, .. }| {
                let expression = if matches!(options, XmlityFieldValueGroupDeriveOpts::Value(XmlityFieldValueDeriveOpts {default: true, ..}) ) {
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
            |DeserializeField {
                 builder_field_ident,
                 field_ident,
                 ..
             }| {
                let expression = parse_quote! {
                    ::xmlity::de::DeserializationGroupBuilder::finish::<<A as ::xmlity::de::AttributesAccess<#visitor_lifetime>>::Error>(#builder_field_ident)?
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
        fields: impl IntoIterator<Item = DeserializeField<FieldIdent, XmlityFieldValueGroupDeriveOpts>>
            + Clone,
        allow_unknown_children: bool,
        order: ElementOrder,
    ) -> Vec<Stmt> {
        let visit = SeqVisitLoop::new(access_ident, allow_unknown_children, order, fields);

        let field_storage = visit.field_storage();
        let access_loop = visit.access_loop();

        parse_quote! {
            #field_storage

            #(#access_loop)*
        }
    }
}

impl VisitorBuilder for SerializeNoneStructBuilder {
    fn visit_seq_fn_body(
        &self,
        ast: &syn::DeriveInput,
        visitor_lifetime: &Lifetime,
        seq_access_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let DeriveInput { data, .. } = ast;

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

                    DeriveResult::Ok(DeserializeField {
                        builder_field_ident: FieldIdent::Named(field_ident.clone()),
                        field_ident: FieldIdent::Named(field_ident),
                        options: XmlityFieldDeriveOpts::from_field(f)?,
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
                        builder_field_ident: FieldIdent::Named(Ident::new(
                            &format!("__{}", i),
                            f.span(),
                        )),
                        field_ident: FieldIdent::Indexed(syn::Index::from(i)),
                        options: XmlityFieldDeriveOpts::from_field(f)?,
                        field_type: f.ty.clone(),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            _ => unreachable!(),
        };

        let element_fields = fields.clone().into_iter().filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                XmlityFieldDeriveOpts::Value(opts) => Some(opts),
                _ => None,
            })
        });

        let group_fields = fields.clone().into_iter().filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                XmlityFieldDeriveOpts::Group(opts) => Some(opts),
                _ => None,
            })
        });

        let getter_declarations = Self::field_decl(element_fields.clone(), group_fields.clone());

        let element_group_fields = fields.clone().into_iter().filter_map(|field| {
            field.map_options_opt(|opt| match opt {
                XmlityFieldDeriveOpts::Value(opts) => {
                    Some(XmlityFieldValueGroupDeriveOpts::Value(opts))
                }
                XmlityFieldDeriveOpts::Group(opts) => {
                    Some(XmlityFieldValueGroupDeriveOpts::Group(opts))
                }
                XmlityFieldDeriveOpts::Attribute(_) => None,
            })
        });

        let children_loop = if element_group_fields.clone().next().is_some() {
            Self::seq_access(
                seq_access_ident,
                element_group_fields,
                false,
                ElementOrder::Loose,
            )
        } else {
            Vec::new()
        };

        let constructor = Self::constructor_expr(
            &ast.ident,
            visitor_lifetime,
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

    fn visitor_definition(&self, ast: &syn::DeriveInput) -> Result<ItemStruct, DeriveError> {
        let DeriveInput {
            ident, generics, ..
        } = ast;
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
}

impl DeserializeBuilder for SerializeNoneStructBuilder {
    fn deserialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        deserializer_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let formatter_expecting = format!("struct {}", ast.ident);

        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());

        let visitor_def = <Self as VisitorBuilder>::visitor_definition(self, ast)?;
        let visitor_trait_impl = <Self as VisitorBuilderExt>::visitor_trait_impl(
            self,
            ast,
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
}

pub struct EnumNoneVisitorBuilder {}

impl EnumNoneVisitorBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

impl VisitorBuilder for EnumNoneVisitorBuilder {
    fn visit_seq_fn_body(
        &self,
        DeriveInput { ident, data, .. }: &DeriveInput,
        _visitor_lifetime: &Lifetime,
        seq_access_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let data_enum = match &data {
            syn::Data::Enum(data_enum) => data_enum,
            _ => panic!("Wrong options. Only enums can be used for xelement."),
        };
        let variants = data_enum.variants.iter().collect::<Vec<_>>();

        let variants = variants.clone().into_iter().map::<Option<ExprIf>, _>(|Variant {
            ident: variant_ident,
            fields: variant_fields,
            ..
        }| {
            match variant_fields {
                syn::Fields::Named(_fields) => {
                    None
                }
                syn::Fields::Unnamed(fields) if fields.unnamed.len() != 1 => {
                    None
                }
                syn::Fields::Unnamed(fields) => {
                    let Field {
                        ty: field_type,
                        ..
                    } = fields.unnamed.first().expect("This is guaranteed by the check above");

                    Some(parse_quote! {
                        if let ::core::result::Result::Ok(::core::option::Option::Some(_v)) = ::xmlity::de::SeqAccess::next_element::<#field_type>(&mut #seq_access_ident) {
                            return ::core::result::Result::Ok(#ident::#variant_ident(_v));
                        }
                    })
                }
                syn::Fields::Unit =>{
                    None
                },
            }
        }).collect::<Option<Vec<_>>>().unwrap_or_default();
        let ident_string = ident.to_string();

        Ok(Some(parse_quote! {
            #(#variants)*

            ::core::result::Result::Err(::xmlity::de::Error::no_possible_variant(#ident_string))
        }))
    }

    fn visitor_definition(&self, ast: &syn::DeriveInput) -> Result<syn::ItemStruct, DeriveError> {
        let DeriveInput {
            ident, generics, ..
        } = ast;
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
}

impl DeserializeBuilder for EnumNoneVisitorBuilder {
    fn deserialize_fn_body(
        &self,
        ast: &syn::DeriveInput,
        deserializer_ident: &Ident,
        _deserialize_lifetime: &Lifetime,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let formatter_expecting = format!("enum {}", ast.ident);

        let visitor_ident = Ident::new("__Visitor", Span::mixed_site());

        let visitor_def = <Self as VisitorBuilder>::visitor_definition(self, ast)?;
        let visitor_trait_impl = <Self as VisitorBuilderExt>::visitor_trait_impl(
            self,
            ast,
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
}
