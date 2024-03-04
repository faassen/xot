use crate::NameId;

/// Indentation: pretty-print XML or HTML.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Indentation {
    /// A list of element names where indentation changes are suppressed.
    pub suppress: Vec<NameId>,
}
