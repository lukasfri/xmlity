use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{parse_quote, Arm, ImplItemFn, ItemImpl, Stmt};
use syn::{DeriveInput, Ident};

use crate::options::{WithExpandedName, XmlityRootAttributeDeriveOpts};
use crate::simple_compile_error;

use crate::DeriveError;
use crate::DeriveMacro;

trait SerializeAttributeBuilder {
    /// Returns the content inside the `Deserialize::deserialize` function.
    fn serialize_attribute_fn_body(
        &self,
        ast: &syn::DeriveInput,
        serializer_access: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError>;
}

trait SerializeAttributeBuilderExt: SerializeAttributeBuilder {
    fn serialize_attribute_fn(&self, ast: &syn::DeriveInput) -> Result<ImplItemFn, DeriveError>;
    fn serialize_attribute_trait_impl(
        &self,
        ast: &syn::DeriveInput,
    ) -> Result<ItemImpl, DeriveError>;
}

impl<T: SerializeAttributeBuilder> SerializeAttributeBuilderExt for T {
    fn serialize_attribute_fn(&self, ast: &syn::DeriveInput) -> Result<ImplItemFn, DeriveError> {
        let serializer_access_ident = Ident::new("__serializer", ast.span());
        let body = self.serialize_attribute_fn_body(ast, &serializer_access_ident)?;
        Ok(parse_quote!(
            fn serialize_attribute<S>(&self, mut #serializer_access_ident: S) -> Result<<S as ::xmlity::AttributeSerializer>::Ok, <S as ::xmlity::AttributeSerializer>::Error>
            where
                S: ::xmlity::AttributeSerializer,
            {
                #(#body)*
            }
        ))
    }

    fn serialize_attribute_trait_impl(
        &self,
        ast @ DeriveInput {
            ident, generics, ..
        }: &syn::DeriveInput,
    ) -> Result<ItemImpl, DeriveError> {
        let serialize_attribute_fn = self.serialize_attribute_fn(ast)?;

        let non_bound_generics = crate::non_bound_generics(generics);

        Ok(parse_quote! {
            impl #generics ::xmlity::SerializeAttribute for #ident #non_bound_generics {
                #serialize_attribute_fn
            }
        })
    }
}

pub struct StructUnnamedSingleFieldAttributeSerializeBuilder<'a> {
    opts: &'a XmlityRootAttributeDeriveOpts,
}

impl<'a> StructUnnamedSingleFieldAttributeSerializeBuilder<'a> {
    pub fn new(opts: &'a XmlityRootAttributeDeriveOpts) -> Self {
        Self { opts }
    }
}

impl SerializeAttributeBuilder for StructUnnamedSingleFieldAttributeSerializeBuilder<'_> {
    fn serialize_attribute_fn_body(
        &self,
        ast: &syn::DeriveInput,
        serializer_access: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let DeriveInput { ident, data, .. } = ast;

        let XmlityRootAttributeDeriveOpts {
            preferred_prefix,
            enforce_prefix,
            ..
        } = self.opts;
        let ident_name = ident.to_string();
        let expanded_name = self.opts.expanded_name(&ident_name);
        let _unnamed_fields = match data {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Unnamed(fields),
                ..
            }) => fields,
            _ => unreachable!(),
        };

        let access_ident = Ident::new("__sa", proc_macro2::Span::call_site());
        let xml_name_temp_ident = Ident::new("__xml_name", proc_macro2::Span::call_site());

        let preferred_prefix_setting = preferred_prefix.as_ref().map::<Stmt, _>(|preferred_prefix| parse_quote! {
            ::xmlity::ser::SerializeAttributeAccess::preferred_prefix(&mut #access_ident, ::core::option::Option::Some(#preferred_prefix))?;
        });
        let enforce_prefix_setting = Some(*enforce_prefix).filter(|&enforce_prefix| enforce_prefix).map::<Stmt, _>(|enforce_prefix| parse_quote! {
            ::xmlity::ser::SerializeAttributeAccess::include_prefix(&mut #access_ident, #enforce_prefix)?;
        });

        Ok(parse_quote! {
            let #xml_name_temp_ident = #expanded_name;
            let mut #access_ident = ::xmlity::AttributeSerializer::serialize_attribute(
                &mut #serializer_access,
                &#xml_name_temp_ident,
            )?;
            #preferred_prefix_setting
            #enforce_prefix_setting
            ::xmlity::ser::SerializeAttributeAccess::end(#access_ident, &self.0.to_string())
        })
    }
}

pub struct EnumSingleFieldAttributeSerializeBuilder {}

impl EnumSingleFieldAttributeSerializeBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

impl SerializeAttributeBuilder for EnumSingleFieldAttributeSerializeBuilder {
    fn serialize_attribute_fn_body(
        &self,
        ast: &syn::DeriveInput,
        serializer_access: &Ident,
    ) -> Result<Vec<Stmt>, DeriveError> {
        let DeriveInput { ident, data, .. } = ast;
        let syn::Data::Enum(syn::DataEnum { variants, .. }) = data else {
            unreachable!()
        };

        let variants = variants
            .iter()
            .map::<Result<Arm, DeriveError>, _>(|variant| {
                let variant_name = &variant.ident;
                match &variant.fields {
                    syn::Fields::Named(_fields) => {
                        Err(DeriveError::custom("Named fields are not supported yet"))
                    }
                    syn::Fields::Unnamed(fields) => {
                        if fields.unnamed.len() == 1 {
                            Ok(parse_quote! {
                                #ident::#variant_name(val) => {
                                    ::xmlity::Serialize::serialize(&val, &mut #serializer_access)
                                },
                            })
                        } else {
                            Err(DeriveError::custom(
                                "Enum variants with more than one field are not supported",
                            ))
                        }
                    }
                    syn::Fields::Unit => {
                        Err(DeriveError::custom("Unit variants are not supported yet"))
                    }
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(parse_quote!(
            match self {
                #(#variants)*
            }
        ))
    }
}

pub struct DeriveSerializeAttribute;

impl DeriveMacro for DeriveSerializeAttribute {
    fn input_to_derive(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, DeriveError> {
        let opts = XmlityRootAttributeDeriveOpts::parse(ast)?.unwrap_or_default();

        match &ast.data {
            syn::Data::Struct(syn::DataStruct { fields, .. }) => match fields {
                syn::Fields::Unnamed(_) => {
                    StructUnnamedSingleFieldAttributeSerializeBuilder::new(&opts)
                        .serialize_attribute_trait_impl(ast)
                        .map(|x| x.to_token_stream())
                }
                syn::Fields::Named(_) | syn::Fields::Unit => {
                    Ok(simple_compile_error("Named fields are not supported yet"))
                }
            },
            syn::Data::Enum(_) => EnumSingleFieldAttributeSerializeBuilder::new()
                .serialize_attribute_trait_impl(ast)
                .map(|x| x.to_token_stream()),
            syn::Data::Union(_) => unreachable!(),
        }
    }
}
