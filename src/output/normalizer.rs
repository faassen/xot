use std::borrow::Cow;

#[cfg(doc)]
use crate::Xot;
#[cfg(feature = "icu")]
#[cfg(doc)]
use icu;

/// A normalizer.
///
/// Normalizes text and attribute values. You can pass a normalizer using
/// [`Xot::serialize_xml_string_with_normalizer`] and
/// [`Xot::serialize_xml_write_with_normalizer`].
///
#[cfg_attr(
    feature = "icu",
    doc = r##"
If you enable the `icu` feature, you can use the `icu` normalizers [`icu::normalizer::ComposingNormalizer`] and
[`icu::normalizer::DecomposingNormalizer`] as normalizers."##
)]
pub trait Normalizer {
    /// Given a piece of text, give back the normalized version.
    fn normalize<'a>(&self, content: Cow<'a, str>) -> Cow<'a, str>;

    // in theory we could use normalize_iter on ICU to avoid allocating a string, but
    // that gets too hairy for now
}

/// A normalizer that does nothing at all. This is used by default if
/// you don't specify a normalizer.
pub struct NoopNormalizer;

impl Normalizer for NoopNormalizer {
    #[inline]
    fn normalize<'a>(&self, content: Cow<'a, str>) -> Cow<'a, str> {
        content
    }
}
