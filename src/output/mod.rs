//! Xot offers functionality to serialize XML data in different ways.
//!
//! This module lets you control serialization in various ways.
mod common;
pub mod html5;
#[cfg(feature = "icu")]
mod icu_normalization;
mod normalizer;
pub mod xml;

pub use common::Indentation;
pub use normalizer::{NoopNormalizer, Normalizer};
