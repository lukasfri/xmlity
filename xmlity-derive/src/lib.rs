//! # XMLity Derive
//!
//! This crate contains the proc-macros for XMLity, specifically the derive macros for [`Serialize`], [`SerializeAttribute`], [`Deserialize`], [`SerializationGroup`], and [`DeserializationGroup`].
//!
//! Each of these macros has its own documentation, which can be found by following the links above.
//!
//! The attributes used by these macros are made to be compatible with their counterparts:
//! - [`Serialize`] and [`SerializeAttribute`] use the same attributes with the same options as [`Deserialize`].
//! - [`SerializationGroup`] use the same attributes with the same options as [`DeserializationGroup`].
//!
//! There are some attributes only used by either serialization or deserialization. These are highlighted in the documentation for each macro.
//!
//! ## Example
//! ```ignore
//! use xmlity_derive::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! #[xelement(name = "name")]
//! struct Name(String);
//!
//! #[derive(Serialize, Deserialize)]
//! #[xelement(name = "age")]
//! struct Age(u8);
//!
//! #[derive(Serialize, Deserialize)]
//! #[xelement(name = "person")]
//! struct Person {
//!     name: Name,
//!     age: Age,
//! }
//! ```
//!
//! The derive macros are re-exported by the `xmlity` crate in the `derive` feature, so you can use them directly from there without referring to [`xmlity_derive`].

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct _ReadMeDocTests;

pub(crate) mod common;
mod de;
mod derive;
mod options;
mod ser;
mod utils;

use de::{DeriveDeserializationGroup, DeriveDeserialize};
use derive::{DeriveError, DeriveMacro, DeriveMacroExt, DeriveResult};
use ser::{DeriveSerializationGroup, DeriveSerialize, DeriveSerializeAttribute};

#[doc = include_str!("./ser/Serialize.md")]
#[proc_macro_derive(Serialize, attributes(xelement, xattribute, xgroup, xvalue))]
pub fn derive_serialize_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    DeriveSerialize::derive(item)
}

#[doc = include_str!("./ser/SerializeAttribute.md")]
#[proc_macro_derive(SerializeAttribute, attributes(xattribute))]
pub fn derive_serialize_attribute_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    DeriveSerializeAttribute::derive(item)
}

#[doc = include_str!("./de/Deserialize.md")]
#[proc_macro_derive(Deserialize, attributes(xelement, xattribute, xgroup, xvalue))]
pub fn derive_deserialize_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    DeriveDeserialize::derive(item)
}

#[doc = include_str!("./ser/SerializationGroup.md")]
#[proc_macro_derive(SerializationGroup, attributes(xvalue, xattribute, xgroup))]
pub fn derive_serialization_group_attribute_fn(
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    DeriveSerializationGroup::derive(item)
}

#[doc = include_str!("./de/DeserializationGroup.md")]
#[proc_macro_derive(DeserializationGroup, attributes(xvalue, xattribute, xgroup))]
pub fn derive_deserialization_group_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    DeriveDeserializationGroup::derive(item)
}
