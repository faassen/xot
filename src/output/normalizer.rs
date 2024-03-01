use std::borrow::Cow;

/// A text normalizer.
///
/// If you enable the `icu` feature.
pub trait Normalizer {
    /// Given a piece of text, give back the normalized version.
    fn normalize(content: &str) -> Cow<str>;
}

/// A normalizer that does nothing at all.
pub struct NoopNormalizer;

impl Normalizer for NoopNormalizer {
    #[inline]
    fn normalize(content: &str) -> Cow<str> {
        content.into()
    }
}
