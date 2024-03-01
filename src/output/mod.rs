#[cfg(feature = "icu")]
mod icu_normalization;
mod normalizer;
/// Xot offers functions to affect the serialization of the output,
/// such as pretty printing and the inclusion of a XML declaration.
pub mod xml;

#[cfg(feature = "icu")]
pub use icu_normalization::NormalizationForm;

pub use normalizer::{NoopNormalizer, Normalizer};
