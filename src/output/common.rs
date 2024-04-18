use crate::NameId;

/// Indentation: pretty-print XML or HTML.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Indentation {
    /// A list of element names where indentation changes are suppressed.
    pub suppress: Vec<NameId>,
}

/// Parameters used when serializing tokens.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TokenSerializeParameters {
    /// Elements that should have their text content be serialized as CDATA
    /// sections.
    pub cdata_section_elements: Vec<NameId>,

    /// Whether to escape the `>` character in text content. By default this is
    /// true, which means that `>` is escaped as `&gt;`. If you set this to true,
    /// `>` is not escaped, except for the special case of `]]>` outside of CDATA,
    /// which is mandated by the XML specification to always be escaped.
    pub unescaped_gt: bool,
}
