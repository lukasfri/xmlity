//! This module contains implementations for common types including primitives, iterators, and common collections, as well as some utility types.
//!
//! It also contains some visitors for the types which can be reused, including [`iterator::IteratorVisitor`].

pub mod common;
pub mod iterator;
mod primitive;
pub mod string;
pub mod utils;
pub mod value;
