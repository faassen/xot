use std::borrow::Cow;

/// A text normalizer.
///
/// If you enable the `icu` feature.
pub trait Normalizer {
    /// Given a piece of text, give back the normalized version.
    fn normalize<'a>(&self, content: Cow<'a, str>) -> Cow<'a, str>;

    // in theory we could use normalize_iter on ICU to avoid allocating a string, but
    // that gets too hairy for now
}

/// A normalizer that does nothing at all.
pub struct NoopNormalizer;

impl Normalizer for NoopNormalizer {
    #[inline]
    fn normalize<'a>(&self, content: Cow<'a, str>) -> Cow<'a, str> {
        content
    }
}
