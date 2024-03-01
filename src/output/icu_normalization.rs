/// Unicode normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizationForm {
    /// Normalization Form C, using the rules specified in [Character Model for
    /// the World Wide Web 1.0:
    /// Normalization](https://www.w3.org/TR/xslt-xquery-serialization/#charmod-norm).
    Nfc,
    /// NFD specifies the serialized result will be in Normalization Form D, as
    /// specified in [UAX #15: Unicode Normalization
    /// Forms](https://www.w3.org/TR/xslt-xquery-serialization/#UNICODE-NORMALIZATION-FORM).
    Nfd,
    /// Normalization Form KC, as specified in [UAX #15: Unicode Normalization
    /// Forms](https://www.w3.org/TR/xslt-xquery-serialization/#UNICODE-NORMALIZATION-FORM).
    Nfkc,
    /// Normalization Form KD, as specified in [UAX #15: Unicode Normalization
    /// Forms](https://www.w3.org/TR/xslt-xquery-serialization/#UNICODE-NORMALIZATION-FORM).
    Nfkd,
    // TODO: fully normalized
}

// pub(crate) fn create_normalize(
//     provider: &impl icu_provider::AnyProvider,
//     normalization_form: Option<output::xml::NormalizationForm>,
// ) -> Result<Normalize, Error> {
//     if let Some(normalization_form) = normalization_form {
//         use crate::output::xml::NormalizationForm::*;
//         use icu::normalizer::{ComposingNormalizer, DecomposingNormalizer};

//         match normalization_form {
//             Nfc => {
//                 let normalizer = ComposingNormalizer::try_new_nfc_with_any_provider(provider)?;
//                 Ok(Box::new(move |content| {
//                     normalizer.normalize(content).into()
//                 }))
//             }
//             Nfd => {
//                 let normalizer = DecomposingNormalizer::try_new_nfd_with_any_provider(provider)?;
//                 Ok(Box::new(move |content| {
//                     normalizer.normalize(content).into()
//                 }))
//             }
//             Nfkc => {
//                 let normalizer = ComposingNormalizer::try_new_nfkc_with_any_provider(provider)?;
//                 Ok(Box::new(move |content| {
//                     normalizer.normalize(content).into()
//                 }))
//             }
//             Nfkd => {
//                 let normalizer = DecomposingNormalizer::try_new_nfkd_with_any_provider(provider)?;
//                 Ok(Box::new(move |content| {
//                     normalizer.normalize(content).into()
//                 }))
//             }
//         }
//     } else {
//         Ok(Box::new(|content| content.into()))
//     }
// }
