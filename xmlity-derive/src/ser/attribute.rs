use quote::quote;
use syn::{DeriveInput, Ident};

use crate::options::XmlityRootAttributeDeriveOpts;
use crate::simple_compile_error;

use crate::DeriveError;
use crate::DeriveMacro;

pub struct SerializeAttributeTraitImplBuilder<'a> {
    ident: &'a Ident,
    generics: &'a syn::Generics,
    serializer_access: &'a Ident,
    implementation: proc_macro2::TokenStream,
}

impl<'a> SerializeAttributeTraitImplBuilder<'a> {
    pub fn new(
        ident: &'a proc_macro2::Ident,
        generics: &'a syn::Generics,
        serializer_access: &'a Ident,
        implementation: proc_macro2::TokenStream,
    ) -> Self {
        Self {
            ident,
            generics,
            serializer_access,
            implementation,
        }
    }

    fn trait_impl(&self) -> proc_macro2::TokenStream {
        let Self {
            ident,
            generics,
            serializer_access,
            implementation,
        } = self;

        let non_bound_generics = crate::non_bound_generics(generics);

        quote! {
            impl #generics ::xmlity::SerializeAttribute for #ident #non_bound_generics {
                fn serialize_attribute<S>(&self, mut #serializer_access: S) -> Result<<S as ::xmlity::AttributeSerializer>::Ok, <S as ::xmlity::AttributeSerializer>::Error>
                where
                    S: ::xmlity::AttributeSerializer,
                {
                    #implementation
                }
            }
        }
    }
}

trait SerializeAttributeContent {
    /// Returns the content inside the `Deserialize::deserialize` function.
    fn serialize_attribute_content(
        &self,
        ast: &syn::DeriveInput,
        serializer_access: &Ident,
    ) -> Result<proc_macro2::TokenStream, DeriveError>;
}

trait SerializeAttributeContentExt: SerializeAttributeContent {
    fn serialize_attribute_impl(
        &self,
        ast: &syn::DeriveInput,
    ) -> Result<proc_macro2::TokenStream, DeriveError>;
}

impl<T: SerializeAttributeContent> SerializeAttributeContentExt for T {
    fn serialize_attribute_impl(
        &self,
        ast @ DeriveInput {
            ident, generics, ..
        }: &syn::DeriveInput,
    ) -> Result<proc_macro2::TokenStream, DeriveError> {
        let serializer_access = Ident::new("__serializer", ident.span());

        let implementation = self.serialize_attribute_content(ast, &serializer_access)?;

        let trait_impl_builder = SerializeAttributeTraitImplBuilder::new(
            ident,
            generics,
            &serializer_access,
            implementation,
        );

        Ok(trait_impl_builder.trait_impl())
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

impl SerializeAttributeContent for StructUnnamedSingleFieldAttributeSerializeBuilder<'_> {
    fn serialize_attribute_content(
        &self,
        ast: &syn::DeriveInput,
        serializer_access: &Ident,
    ) -> Result<proc_macro2::TokenStream, DeriveError> {
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

        let preferred_prefix_setting = preferred_prefix.0.as_ref().map(|preferred_prefix| quote! {
            ::xmlity::ser::SerializeAttributeAccess::preferred_prefix(&mut #access_ident, ::core::option::Option::Some(::xmlity::Prefix::new(#preferred_prefix).expect("XML prefix in derive macro is invalid. This is a bug in xmlity. Please report it.")))?;
        });
        let enforce_prefix_setting = Some(*enforce_prefix).filter(|&enforce_prefix| enforce_prefix).map(|enforce_prefix| quote! {
            ::xmlity::ser::SerializeAttributeAccess::include_prefix(&mut #access_ident, #enforce_prefix)?;
        });

        Ok(quote! {
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

pub struct EnumSingleFieldAttributeSerializeBuilder<'a> {
    opts: &'a XmlityRootAttributeDeriveOpts,
}

impl<'a> EnumSingleFieldAttributeSerializeBuilder<'a> {
    pub fn new(opts: &'a XmlityRootAttributeDeriveOpts) -> Self {
        Self { opts }
    }
}

impl SerializeAttributeContent for EnumSingleFieldAttributeSerializeBuilder<'_> {
    fn serialize_attribute_content(
        &self,
        ast: &syn::DeriveInput,
        serializer_access: &Ident,
    ) -> Result<proc_macro2::TokenStream, DeriveError> {
        let DeriveInput { ident, data, .. } = ast;
        let syn::Data::Enum(syn::DataEnum { variants, .. }) = data else {
            unreachable!()
        };

        let variants = variants.iter().map(|variant| {
            let variant_name = &variant.ident;
            match &variant.fields {
                syn::Fields::Named(_fields) => {
                    simple_compile_error("Named fields are not supported yet")
                }
                syn::Fields::Unnamed(fields) => {
                    if fields.unnamed.len() == 1 {
                        quote! {
                            #ident::#variant_name(val) => {
                                ::xmlity::Serialize::serialize(&val, &mut #serializer_access)
                            },
                        }
                    } else {
                        simple_compile_error(
                            "Enum variants with more than one field are not supported",
                        )
                    }
                }
                syn::Fields::Unit => simple_compile_error("Unit variants are not supported yet"),
            }
        });

        Ok(quote! {
            match self {
                #(#variants)*
            }
        })
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
                        .serialize_attribute_impl(ast)
                }
                syn::Fields::Named(_) | syn::Fields::Unit => {
                    Ok(simple_compile_error("Named fields are not supported yet"))
                }
            },
            syn::Data::Enum(_) => {
                EnumSingleFieldAttributeSerializeBuilder::new(&opts).serialize_attribute_impl(ast)
            }
            syn::Data::Union(_) => unreachable!(),
        }
    }
}
