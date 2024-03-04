//! html5 output method.
//!
//! The main entry point is [`Parameters`], which you can pass into various
//! serialization methods to control the output.
//!

use crate::NameId;

use super::Indentation;

/// Parameters for HTML generation.
///
/// This only supports HTML 5.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Parameters {
    /// Pretty-print HTML, and a list of elements where this is suppressed.
    pub indentation: Option<Indentation>,
    /// Elements that should be serialized as CDATA sections.
    ///
    /// These should only be used in non-XML content, like MathML or SVG.
    pub cdata_section_elements: Vec<NameId>,

    /// If set, this causes a meta element to be added (or updated) with
    /// http-equiv="Content-Type" and the given content type.
    pub media_type: Option<String>,
    // TODO: character maps
}
