//! html5 output method.
//!
//! The main entry point is [`Parameters`], which you can pass into various
//! serialization methods to control the output.
//!
//! You can use these parameters to control the HTML5 serialization, which you
//! can obtain using [`Xot::html5`].

use crate::NameId;

use super::Indentation;

/// Parameters for HTML generation.
///
/// This only supports HTML 5.
///
/// This follows the HTML5 serialization rules described in
/// https://www.w3.org/TR/xslt-xquery-serialization/
///
/// In summary:
///
/// - no-namespace and XHTML namespace is serialized as HTML tags
///
/// - Always use explicit close tags such as `</p>`, never use self-closing
///  tags such as `<p/>`.
///
/// - certain HTML tags are unclosed (void names such `br`, `meta`), i.e.
///   `<br>`
///
/// - mathml and svg are serialized with default namespace only, no prefix.
///
/// - certain HTML tags are rendered preformatted (`pre`, `script`)
///
/// - Indentation takes inline elements (phrasing content names such as `a`,
///   `span`, etc) into account. U
///
/// - Unknown HTML elements (that have no namespace or in the XHTML namespace) are
///   also treated as inline elements.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Parameters {
    /// Pretty-print HTML, and a list of elements where this is suppressed.
    ///
    /// This recognizes inline (phrasing) elements.
    pub indentation: Option<Indentation>,
    /// Elements that should be serialized as CDATA sections.
    ///
    /// These should only be used for elements in non-XML content, like MathML
    /// or SVG.
    pub cdata_section_elements: Vec<NameId>,
    // TODO: character maps
}
