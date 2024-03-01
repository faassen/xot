//! Xot offers functionality to serialize XML data in different ways.
//!
//! This module lets you control serialization in various ways.
#[cfg(feature = "icu")]
mod icu_normalization;
mod normalizer;
pub mod xml;

pub use normalizer::{NoopNormalizer, Normalizer};
