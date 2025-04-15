use proc_macro2::Span;
use syn::{
    parse_quote, Data, DataStruct, DeriveInput, Expr, ExprIf, Field, Ident, ItemStruct, Lifetime,
    LifetimeParam, Stmt, Variant,
};

use crate::{
    de::{
        common::{DeserializeBuilder, VisitorBuilder, VisitorBuilderExt},
        constructor_expr, StructType,
    },
    DeriveError, DeserializeBuilderField,
};

pub struct SerializeNoneStructBuilder;

impl SerializeNoneStructBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

impl VisitorBuilder for SerializeNoneStructBuilder {
    fn visit_seq_fn_body(
        &self,
        ast: &syn::DeriveInput,
        _visitor_lifetime: &Lifetime,
        seq_access_ident: &Ident,
    ) -> Result<Option<Vec<Stmt>>, DeriveError> {
        let Data::Struct(DataStruct { fields, .. }) = &ast.data else {
            unreachable!()
        };

        let constructor_type = match &fields {
            syn::Fields::Named(_) => StructType::Named,
            syn::Fields::Unnamed(_) => StructType::Unnamed,
            _ => unreachable!(),
        };

        let fields = crate::de::fields(ast)?.into_iter()
        .map::<(_, Expr), _>(|DeserializeBuilderField { field_ident,  field_type, .. }| {

            (field_ident, parse_quote! {
                ::core::option::Option::ok_or_else(
                    ::xmlity::de::SeqAccess::next_element_seq::<#field_type>(&mut #seq_access_ident)?,
                    ::xmlity::de::Error::missing_data,
                )?
            })
        });

        let constructor = constructor_expr(&ast.ident, fields, &constructor_type);

        Ok(Some(parse_quote! {
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

            ::xmlity::de::Deserializer::deserialize_any(#deserializer_ident, #visitor_ident {
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
