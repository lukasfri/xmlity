use quote::quote;
use syn::{DataEnum, DataStruct, DeriveInput, Ident};

use crate::simple_compile_error;

use crate::ExpandedName;

fn derive_unnamed_struct_serialize(
    expanded_name: &ExpandedName,
    preferred_prefix: Option<&str>,
    enforce_prefix: bool,
    fields: &syn::FieldsUnnamed,
) -> proc_macro2::TokenStream {
    if fields.unnamed.len() == 1 {
        let access_ident = Ident::new("__sa", proc_macro2::Span::call_site());
        let xml_name_temp_ident = Ident::new("__xml_name", proc_macro2::Span::call_site());

        let preferred_prefix_setting = preferred_prefix.map(|preferred_prefix| quote! {
            ::xmlity::ser::SerializeAttributeAccess::preferred_prefix(&mut #access_ident, ::core::option::Option::Some(::xmlity::Prefix::new(#preferred_prefix).expect("XML prefix in derive macro is invalid. This is a bug in xmlity. Please report it.")))?;
        });
        let enforce_prefix_setting = Some(enforce_prefix).filter(|&enforce_prefix| enforce_prefix).map(|enforce_prefix| quote! {
            ::xmlity::ser::SerializeAttributeAccess::include_prefix(&mut #access_ident, #enforce_prefix)?;
        });

        quote! {
            let #xml_name_temp_ident = #expanded_name;
            let mut #access_ident = ::xmlity::AttributeSerializer::serialize_attribute(
                &mut serializer,
                &#xml_name_temp_ident,
            )?;
            #preferred_prefix_setting
            #enforce_prefix_setting
            ::xmlity::ser::SerializeAttributeAccess::end(#access_ident, &self.0.to_string())
        }
    } else {
        simple_compile_error("Unnamed structs with more than one field are not supported")
    }
}

fn derive_enum_serialize(
    ident: &syn::Ident,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
) -> proc_macro2::TokenStream {
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
                            ::xmlity::Serialize::serialize(&val, &mut serializer)
                        },
                    }
                } else {
                    simple_compile_error("Enum variants with more than one field are not supported")
                }
            }
            syn::Fields::Unit => simple_compile_error("Unit variants are not supported yet"),
        }
    });
    quote! {
        match self {
            #(#variants)*
        }
    }
}

fn serialize_trait_impl(
    ident: &proc_macro2::Ident,
    generics: &syn::Generics,
    implementation: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let non_bound_generics = crate::non_bound_generics(generics);

    quote! {
        impl #generics ::xmlity::SerializeAttribute for #ident #non_bound_generics {
            fn serialize_attribute<S>(&self, mut serializer: S) -> Result<<S as ::xmlity::AttributeSerializer>::Ok, <S as ::xmlity::AttributeSerializer>::Error>
            where
                S: ::xmlity::AttributeSerializer,
            {
                #implementation
            }
        }
    }
}

pub fn derive_serialize_fn(
    ast: DeriveInput,
    opts: crate::XmlityRootAttributeDeriveOpts,
) -> proc_macro2::TokenStream {
    let ident_name = ast.ident.to_string();
    let expanded_name = ExpandedName::new(
        opts.name
            .as_ref()
            .map(|a| a.0.as_str())
            .unwrap_or(&ident_name),
        opts.namespace.as_ref().map(|a| a.0.as_str()),
    );

    let preferred_prefix = opts.preferred_prefix.as_ref().map(|a| a.0.as_str());

    let implementation = match &ast.data {
        syn::Data::Struct(DataStruct { fields, .. }) => match fields {
            syn::Fields::Unnamed(fields) => derive_unnamed_struct_serialize(
                &expanded_name,
                preferred_prefix,
                opts.enforce_prefix,
                fields,
            ),
            syn::Fields::Named(_) | syn::Fields::Unit => {
                simple_compile_error("Named fields are not supported yet")
            }
        },
        syn::Data::Enum(DataEnum { variants, .. }) => derive_enum_serialize(&ast.ident, variants),
        syn::Data::Union(_) => unreachable!(),
    };

    serialize_trait_impl(&ast.ident, &ast.generics, implementation)
}
