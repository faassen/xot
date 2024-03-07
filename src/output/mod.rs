//! Xot offers functionality to serialize XML data in different ways.
//!
//! This module lets you control serialization in various ways.
mod common;
mod fullname;
pub mod html5;
mod html5_serializer;
#[cfg(feature = "icu")]
mod icu_normalization;
mod normalizer;
mod pretty;
mod serializer;
pub mod xml;
mod xml_serializer;

pub use common::Indentation;
pub(crate) use fullname::FullnameSerializer;
pub(crate) use fullname::NamespaceDeclarations;
pub(crate) use html5_serializer::{Html5Elements, Html5Serializer};
pub use normalizer::{NoopNormalizer, Normalizer};
pub(crate) use pretty::Pretty;
pub use pretty::PrettyOutputToken;
pub(crate) use serializer::gen_outputs;
pub use serializer::{Output, OutputToken};
pub(crate) use xml_serializer::XmlSerializer;
