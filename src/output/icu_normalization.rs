use std::borrow::Cow;

use icu::normalizer::{ComposingNormalizer, DecomposingNormalizer};

use super::Normalizer;

impl Normalizer for ComposingNormalizer {
    fn normalize<'a>(&self, content: Cow<'a, str>) -> Cow<'a, str> {
        self.normalize(content.as_ref()).into()
    }
}

impl Normalizer for DecomposingNormalizer {
    fn normalize<'a>(&self, content: Cow<'a, str>) -> Cow<'a, str> {
        self.normalize(content.as_ref()).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::{serialize_attribute, serialize_cdata, serialize_text};

    use super::*;

    #[test]
    fn test_icu_normalizer_text() {
        let normalizer = ComposingNormalizer::new_nfc();
        // example taken from https://www.unicode.org/reports/tr15/ figure 5, second example
        let s = serialize_text("\u{1E0B}\u{0323}".into(), &normalizer, false);
        assert_eq!(s, "\u{1E0D}\u{0307}");
    }

    #[test]
    fn test_icu_normalizer_cdata() {
        let normalizer = ComposingNormalizer::new_nfc();
        // example taken from https://www.unicode.org/reports/tr15/ figure 5, second example
        let s = serialize_cdata("\u{1E0B}\u{0323}".into(), &normalizer);
        assert_eq!(s, "<![CDATA[\u{1E0D}\u{0307}]]>");
    }

    #[test]
    fn test_icu_normalizer_attribute() {
        let normalizer = ComposingNormalizer::new_nfc();
        // example taken from https://www.unicode.org/reports/tr15/ figure 5, second example
        let s = serialize_attribute("\u{1E0B}\u{0323}".into(), &normalizer);
        assert_eq!(s, "\u{1E0D}\u{0307}");
    }
}
