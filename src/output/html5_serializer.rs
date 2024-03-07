use std::borrow::Cow;
use std::io;

use ahash::{HashSet, HashSetExt};

use crate::entity::{serialize_attribute, serialize_cdata, serialize_text};
use crate::error::Error;
use crate::id::NameId;
use crate::output::Normalizer;
use crate::xotdata::{Node, Xot};
use crate::NamespaceId;

use super::fullname::FullnameSerializer;
use super::{Output, OutputToken, Pretty};

// used to determine whether something is a HTML 5 element
const XHTML_NS: &str = "https://www.w3.org/1999/xhtml";
const MATHML_NS: &str = "http://www.w3.org/1998/Math/MathML";
const SVG_NS: &str = "http://www.w3.org/2000/svg";

#[derive(Debug)]
pub(crate) struct Html5Elements {
    xhtml_namespace_id: NamespaceId,
    mathml_namespace_id: NamespaceId,
    svg_namespace_id: NamespaceId,
    phrasing_content_names: HtmlNames,
    void_names: HtmlNames,
    formatted_names: HtmlNames,
    no_escape_names: HtmlNames,
}

#[derive(Debug)]
struct HtmlNames {
    xhtml_namespace_id: NamespaceId,
    ids: HashSet<NameId>,
    names: HashSet<String>,
}

impl HtmlNames {
    fn new(xot: &mut Xot, xhtml_namespace_id: NamespaceId, names: &[&str]) -> Self {
        let mut ids = HashSet::new();
        for name in names {
            // lowercase names, no namespace
            ids.insert(xot.add_name_ns(name, xot.no_namespace()));
            // uppercase names, no namespace
            ids.insert(xot.add_name_ns(&name.to_ascii_uppercase(), xot.no_namespace()));
            // lowercase names, XHTML namespace
            ids.insert(xot.add_name_ns(name, xhtml_namespace_id));
            // uppercase names, XHTML namespace
            ids.insert(xot.add_name_ns(&name.to_ascii_uppercase(), xhtml_namespace_id));
        }
        Self {
            xhtml_namespace_id,
            ids,
            names: names.iter().map(|name| name.to_string()).collect(),
        }
    }

    fn is_html_element(&self, xot: &Xot, name_id: NameId) -> bool {
        let namespace = xot.namespace_for_name(name_id);
        namespace == self.xhtml_namespace_id || namespace == xot.no_namespace()
    }

    fn matches(&self, xot: &Xot, name_id: NameId) -> bool {
        // if we match any of the known ids, we're done right away
        if self.ids.contains(&name_id) {
            return true;
        }
        // if this is not an HTML element, we know it isn't one of the HTML names
        if !self.is_html_element(xot, name_id) {
            return false;
        }
        // otherwise, we do a case-insensitive lookup of the local name
        let name = xot.local_name_str(name_id);
        // now lowercase the name and look it up
        let name = name.to_ascii_lowercase();
        self.names.contains(&name)
    }
}

impl Html5Elements {
    pub(crate) fn new(xot: &mut Xot) -> Self {
        let void_names = [
            "area", "base", "br", "col", "embed", "hr", "img", "input", "keygen", "link", "meta",
            "param", "source", "track", "wbr",
            // extra elements not in the HTML5 spec but null in HTML 4
            "basefont", "frame", "isindex",
        ];
        let xhtml_namespace_id = xot.add_namespace(XHTML_NS);
        let mathml_namespace_id = xot.add_namespace(MATHML_NS);
        let svg_namespace_id = xot.add_namespace(SVG_NS);

        let void_names = HtmlNames::new(xot, xhtml_namespace_id, &void_names);

        let phrasing_content_names = [
            "a", "abbr", "area", "audio", "b", "bdi", "bdo", "br", "button", "canvas", "cite",
            "code", "command", "datalist", "del", "dfn", "em", "embed", "i", "iframe", "img",
            "input", "ins", "kbd", "keygen", "label", "map", "mark", "math", "meter", "noscript",
            "object", "output", "progress", "q", "ruby", "s", "samp", "script", "select", "small",
            "span", "strong", "sub", "sup", "svg", "textarea", "time", "u", "var", "video", "wbr",
        ];
        let phrasing_content_names =
            HtmlNames::new(xot, xhtml_namespace_id, &phrasing_content_names);

        let formatted_names = ["pre", "script", "style", "title", "textarea"];
        let formatted_names = HtmlNames::new(xot, xhtml_namespace_id, &formatted_names);

        let no_escape_names = ["script", "style"];
        let no_escape_names = HtmlNames::new(xot, xhtml_namespace_id, &no_escape_names);
        Self {
            xhtml_namespace_id,
            mathml_namespace_id,
            svg_namespace_id,
            void_names,
            phrasing_content_names,
            formatted_names,
            no_escape_names,
        }
    }

    fn is_html_element(&self, xot: &Xot, name_id: NameId) -> bool {
        let namespace = xot.namespace_for_name(name_id);
        self.is_html_namespace(xot, namespace)
    }

    fn must_be_serialized_unprefixed(&self, namespace: NamespaceId) -> bool {
        namespace == self.xhtml_namespace_id
            || namespace == self.mathml_namespace_id
            || namespace == self.svg_namespace_id
    }

    fn is_html_namespace(&self, xot: &Xot, namespace_id: NamespaceId) -> bool {
        namespace_id == self.xhtml_namespace_id || namespace_id == xot.no_namespace()
    }
}

pub(crate) struct Html5Serializer<'a, N: Normalizer> {
    xot: &'a Xot,
    html5_elements: &'a Html5Elements,
    cdata_section_names: &'a [NameId],
    fullname_serializer: FullnameSerializer<'a>,
    normalizer: N,
}

fn html_matches_suppress(
    xot: &Xot,
    html5_elements: &Html5Elements,
    names: &[NameId],
    name_id: NameId,
) -> bool {
    for suppress_name in names {
        if name_id == *suppress_name {
            return true;
        }
        let suppress_name_ns = xot.namespace_for_name(*suppress_name);
        // if it's not in the xhtml namespace or in no namespace, they
        // can't possibly compare anymore
        if !html5_elements.is_html_namespace(xot, suppress_name_ns) {
            return false;
        }
        let name_ns = xot.namespace_for_name(name_id);
        if !html5_elements.is_html_namespace(xot, name_ns) {
            return false;
        }
        // now we can do a case insensitive compare of the local name
        let suppress_name = xot.local_name_str(*suppress_name).to_ascii_lowercase();
        let name = xot.local_name_str(name_id).to_ascii_lowercase();
        if suppress_name == name {
            return true;
        }
    }
    false
}

impl<'a, N: Normalizer> Html5Serializer<'a, N> {
    pub(crate) fn new(
        xot: &'a Xot,
        html5_elements: &'a Html5Elements,
        node: Node,
        cdata_section_names: &'a [NameId],
        normalizer: N,
    ) -> Self {
        let extra_declarations = xot.namespaces_in_scope(node).collect();
        let fullname_serializer = FullnameSerializer::new(xot, extra_declarations);
        Self {
            xot,
            html5_elements,
            cdata_section_names,
            fullname_serializer,
            normalizer,
        }
    }

    pub(crate) fn serialize<W: io::Write>(
        &mut self,
        w: &mut W,
        outputs: impl Iterator<Item = (Node, Output<'a>)>,
    ) -> Result<(), Error> {
        for (node, output) in outputs {
            self.serialize_node(w, node, output)?;
        }
        Ok(())
    }

    pub(crate) fn serialize_pretty<W: io::Write>(
        &mut self,
        w: &mut W,
        outputs: impl Iterator<Item = (Node, Output<'a>)>,
        suppress: &[NameId],
    ) -> Result<(), Error> {
        // we have to do the relatively slow html_matches_suppress call here,
        // as we cannot make an efficient HtmlNames at this point (as this
        // needs a mutable Xot)
        let is_suppressed = |name_id| {
            self.html5_elements
                .formatted_names
                .matches(self.xot, name_id)
                || html_matches_suppress(self.xot, self.html5_elements, suppress, name_id)
        };
        let is_inline = |name_id| {
            self.html5_elements
                .phrasing_content_names
                .matches(self.xot, name_id)
        };
        let mut pretty = Pretty::new(self.xot, is_suppressed, is_inline);
        for (node, output) in outputs {
            let (indentation, newline) = pretty.prettify(node, &output);
            if indentation > 0 {
                w.write_all(" ".repeat(indentation * 2).as_bytes())?;
            }
            self.serialize_node(w, node, output)?;
            if newline {
                w.write_all(b"\n")?;
            }
        }
        Ok(())
    }

    pub(crate) fn serialize_node<W: io::Write>(
        &mut self,
        w: &mut W,
        node: Node,
        output: Output<'a>,
    ) -> Result<(), Error> {
        let data = self.render_output(node, &output)?;
        if data.space {
            w.write_all(b" ").unwrap();
        }
        w.write_all(data.text.as_bytes()).unwrap();
        Ok(())
    }

    pub(crate) fn render_output(
        &mut self,
        node: Node,
        output: &Output<'a>,
    ) -> Result<OutputToken, Error> {
        use Output::*;
        let r = match output {
            StartTagOpen(element) => {
                self.fullname_serializer
                    .push(self.xot.namespace_declarations(node));
                let namespace_id = self.xot.namespace_for_name(element.name_id);
                if self
                    .html5_elements
                    .must_be_serialized_unprefixed(namespace_id)
                    && !self.fullname_serializer.has_empty_prefix(namespace_id)
                {
                    // add the empty prefix for the namespace
                    self.fullname_serializer.add_empty_prefix(namespace_id);
                    // we also need to serialize the additional xmlns
                    let local_name = self.xot.local_name_str(element.name_id);
                    let namespace_uri = self.xot.namespace_str(namespace_id);
                    return Ok(OutputToken {
                        space: false,
                        text: format!("<{} xmlns=\"{}\"", local_name, namespace_uri),
                    });
                }
                OutputToken {
                    space: false,
                    text: format!(
                        "<{}",
                        self.fullname_serializer.element_fullname(element.name_id)?
                    ),
                }
            }
            StartTagClose => OutputToken {
                space: false,
                text: ">".to_string(),
            },
            EndTag(element) => {
                let r = if self
                    .html5_elements
                    .void_names
                    .matches(self.xot, element.name())
                {
                    // void elements don't get their end tag, so we just emit an
                    // empty string
                    OutputToken {
                        space: false,
                        text: "".to_string(),
                    }
                } else {
                    OutputToken {
                        space: false,
                        text: format!(
                            "</{}>",
                            self.fullname_serializer.element_fullname(element.name_id)?
                        ),
                    }
                };
                self.fullname_serializer
                    .pop(self.xot.has_namespace_declarations(node));
                r
            }
            Prefix(prefix_id, namespace_id) => {
                let element_name = self.xot.element(node).unwrap().name();
                // we don't want to output non-empty prefixes unless the
                // element has an attribute with the same prefix
                if namespace_id == &self.xot.xml_namespace()
                    || (*prefix_id == self.xot.empty_prefix()
                        && self.xot.namespace_for_name(element_name) != *namespace_id)
                    || (*prefix_id != self.xot.empty_prefix()
                        && self
                            .html5_elements
                            .must_be_serialized_unprefixed(*namespace_id)
                        && !self
                            .xot
                            .attributes(node)
                            .keys()
                            .any(|name| self.xot.namespace_for_name(name) == *namespace_id))
                {
                    return Ok(OutputToken {
                        space: false,
                        text: "".to_string(),
                    });
                }

                let namespace = self.xot.namespace_str(*namespace_id);
                if *prefix_id == self.xot.empty_prefix_id {
                    OutputToken {
                        space: true,
                        text: format!("xmlns=\"{}\"", namespace),
                    }
                } else {
                    let prefix = self.xot.prefix_str(*prefix_id);
                    OutputToken {
                        space: true,
                        text: format!("xmlns:{}=\"{}\"", prefix, namespace),
                    }
                }
            }
            Attribute(name_id, value) => {
                let fullname = self.fullname_serializer.attribute_fullname(*name_id)?;
                let namespace = self.xot.namespace_for_name(*name_id);
                if self.html5_elements.is_html_namespace(self.xot, namespace) {
                    let local_name = self.xot.local_name_str(*name_id);
                    // boolean attribute
                    // no prefix and local name is the same as value
                    if self
                        .fullname_serializer
                        .attribute_prefix(*name_id)?
                        .is_none()
                        && local_name.to_ascii_lowercase() == value.to_ascii_lowercase()
                    {
                        return Ok(OutputToken {
                            space: true,
                            text: format!("{}", fullname),
                        });
                    }
                }
                let value = if namespace != self.xot.no_namespace() {
                    serialize_attribute((*value).into(), &self.normalizer)
                } else {
                    serialize_attribute_html((*value).into(), &self.normalizer)
                };
                OutputToken {
                    space: true,
                    text: format!("{}=\"{}\"", fullname, value),
                }
            }
            Text(text) => {
                // a text node is always a child of an element
                let parent = self.xot.parent(node).unwrap();
                let element = self.xot.element(parent).unwrap();
                let value = if self
                    .html5_elements
                    .no_escape_names
                    .matches(self.xot, element.name())
                {
                    serialize_text_no_escape((*text).into(), &self.normalizer).to_string()
                } else if self.cdata_section_names.contains(&element.name()) {
                    serialize_cdata((*text).into(), &self.normalizer).to_string()
                } else if self
                    .html5_elements
                    .is_html_element(self.xot, element.name())
                {
                    serialize_text_html((*text).into(), &self.normalizer).to_string()
                } else {
                    serialize_text((*text).into(), &self.normalizer).to_string()
                };
                OutputToken {
                    space: false,
                    text: value,
                }
            }
            Comment(text) => OutputToken {
                space: false,
                text: format!("<!--{}-->", text),
            },
            ProcessingInstruction(target, data) => {
                let (target, ns) = self.xot.name_ns_str(*target);
                if !ns.is_empty() {
                    return Err(Error::NamespaceInProcessingInstruction);
                }
                // for some reason the HTML output method does allow processing
                // instructions, but they don't end with ?>
                if let Some(data) = data {
                    if data.contains('>') {
                        return Err(Error::ProcessingInstructionGtInHtml(data.to_string()));
                    }
                    OutputToken {
                        space: false,
                        text: format!("<?{} {}>", target, data),
                    }
                } else {
                    OutputToken {
                        space: false,
                        text: format!("<?{}>", target),
                    }
                }
            }
        };
        Ok(r)
    }
}

pub(crate) fn serialize_text_html<'a, N: Normalizer>(
    content: Cow<'a, str>,
    normalizer: &N,
) -> Cow<'a, str> {
    let mut result = String::new();
    let mut change = false;
    // if we had normalized_iter on the trait we avoid this string allocation
    let normalized_content = normalizer.normalize(content);
    for c in normalized_content.chars() {
        match c {
            '&' => {
                change = true;
                result.push_str("&amp;")
            }
            '<' => {
                change = true;
                result.push_str("&lt;")
            }
            // non-breaking space
            '\u{a0}' => {
                change = true;
                result.push_str("&nbsp;")
            }
            _ => result.push(c),
        }
    }

    if !change {
        normalized_content
    } else {
        result.into()
    }
}

pub(crate) fn serialize_attribute_html<'a, N: Normalizer>(
    content: Cow<'a, str>,
    normalizer: &N,
) -> Cow<'a, str> {
    let mut result = String::new();
    let mut change = false;
    let normalized_content = normalizer.normalize(content);
    for c in normalized_content.chars() {
        match c {
            '&' => {
                change = true;
                result.push_str("&amp;")
            }
            '\'' => {
                change = true;
                result.push_str("&apos;")
            }
            '"' => {
                change = true;
                result.push_str("&quot;")
            }
            // non-breaking space
            '\u{a0}' => {
                change = true;
                result.push_str("&nbsp;")
            }
            _ => result.push(c),
        }
    }

    if !change {
        normalized_content
    } else {
        result.into()
    }
}

pub(crate) fn serialize_text_no_escape<'a, N: Normalizer>(
    content: Cow<'a, str>,
    normalizer: &N,
) -> Cow<'a, str> {
    normalizer.normalize(content)
}

#[cfg(test)]
mod tests {
    use crate::output::{html5::Parameters, Indentation};

    use super::*;

    #[test]
    fn test_never_empty_html() {
        let mut xot = Xot::new();
        let root = xot
            .parse("<html><head></head><body></body></html>")
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(s, "<!DOCTYPE html><html><head></head><body></body></html>");
    }

    #[test]
    fn test_never_empty_xml_element() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<html><head><foo xmlns="foo"><bar></bar></foo></head><body></body></html>"#)
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html><head><foo xmlns="foo"><bar></bar></foo></head><body></body></html>"#
        );
    }

    #[test]
    fn test_void_element() {
        let mut xot = Xot::new();
        let root = xot.parse("<html><body>foo<br/>bar</body></html>").unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(s, "<!DOCTYPE html><html><body>foo<br>bar</body></html>");
    }

    #[test]
    fn test_escaping_for_normal_content() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<html><head><title>foo &amp; bar</title></head><body>foo &amp; bar</body></html>"#)
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html><head><title>foo &amp; bar</title></head><body>foo &amp; bar</body></html>"#
        );
    }
    #[test]
    fn test_no_escaping_for_script_and_style() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<html><head><script>if (a &lt; b) foo()</script><style>a &lt; b</style></head><body></body></html>"#)
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html><head><script>if (a < b) foo()</script><style>a < b</style></head><body></body></html>"#
        );
    }

    #[test]
    fn test_processing_instruction() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<html><head><?foo bar?></head><body></body></html>"#)
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html><head><?foo bar></head><body></body></html>"#
        );
    }

    #[test]
    fn test_processing_instruction_no_gt() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<html><head><?foo >bar?></head><body></body></html>"#)
            .unwrap();
        let e = xot.html5().to_string(root).unwrap_err();

        assert!(matches!(e, Error::ProcessingInstructionGtInHtml(_)));
        match e {
            Error::ProcessingInstructionGtInHtml(s) => {
                assert_eq!(s, ">bar");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_serialize_text_nbsp() {
        let mut xot = Xot::new();
        let root = xot
            .parse("<html><body>foo\u{00a0}bar</body></html>")
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(s, "<!DOCTYPE html><html><body>foo&nbsp;bar</body></html>");
    }

    #[test]
    fn test_serialize_text_no_nbsp_for_xml_island() {
        let mut xot = Xot::new();
        let root = xot
            .parse("<html><body><island xmlns=\"island\">\u{00a0}</island></body></html>")
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(
            s,
            "<!DOCTYPE html><html><body><island xmlns=\"island\">\u{00a0}</island></body></html>"
        );
    }

    #[test]
    fn test_serialize_attribute_nbsp() {
        let mut xot = Xot::new();
        let root = xot
            .parse("<html><body foo='\u{00a0}'>bar</body></html>")
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html><body foo="&nbsp;">bar</body></html>"#
        );
    }

    #[test]
    fn test_serialize_attribute_nbsp_not_in_prefixed_attribute() {
        let mut xot = Xot::new();
        let root = xot
            .parse("<html><body xmlns:prefix='ns' prefix:foo='\u{00a0}'>bar</body></html>")
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(
            s,
            "<!DOCTYPE html><html><body xmlns:prefix=\"ns\" prefix:foo=\"\u{00a0}\">bar</body></html>".to_string()
        );
    }

    #[test]
    fn test_serialize_attribute_dont_escape_lt() {
        let mut xot = Xot::new();
        let root = xot
            .parse("<html><body foo='&lt;'>bar</body></html>")
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(s, r#"<!DOCTYPE html><html><body foo="<">bar</body></html>"#);
    }

    #[test]
    fn test_serialize_attribute_boolean() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<html><body><option selected="selected"/></body></html>"#)
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html><body><option selected></option></body></html>"#
        );
    }

    #[test]
    fn test_serialize_attribute_boolean_not_when_prefixed() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<html><body><option xmlns:prefix="ns" prefix:selected="prefix:selected"/></body></html>"#)
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html><body><option xmlns:prefix="ns" prefix:selected="prefix:selected"></option></body></html>"#
        );
    }

    #[test]
    fn test_serialize_attribute_boolean_with_prefix() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<html><body><option xmlns:foo="foo" foo:selected="selected"/></body></html>"#)
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html><body><option xmlns:foo="foo" foo:selected="selected"></option></body></html>"#
        );
    }

    #[test]
    fn test_serialize_attribute_boolean_with_xhtml_prefix() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<html><body><option xmlns:foo="https://www.w3.org/1999/xhtml" foo:selected="selected"/></body></html>"#)
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html><body><option xmlns:foo="https://www.w3.org/1999/xhtml" foo:selected="selected"></option></body></html>"#
        );
    }

    #[test]
    fn test_serialize_attribute_boolean_case_insensitive() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<html><body><option selected="SeLecTed"/></body></html>"#)
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html><body><option selected></option></body></html>"#
        );
    }

    #[test]
    fn test_html_no_xml_namespace() {
        let mut xot = Xot::new();
        // note that the serialization spec illegitimately places the XML namespace in
        // https, but it's in http
        let root = xot
            .parse(r#"<html xmlns:xml="http://www.w3.org/XML/1998/namespace"></html>"#)
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(s, "<!DOCTYPE html><html></html>");
    }

    #[test]
    fn test_xhtml_namespace_without_prefix() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<prefix:html xmlns:prefix="https://www.w3.org/1999/xhtml"></prefix:html>"#)
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html xmlns="https://www.w3.org/1999/xhtml"></html>"#
        );
    }

    #[test]
    fn test_xhtml_namespace_without_prefix_but_with_attribute() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<prefix:html xmlns:prefix="https://www.w3.org/1999/xhtml" prefix:a="A"></prefix:html>"#)
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html xmlns="https://www.w3.org/1999/xhtml" xmlns:prefix="https://www.w3.org/1999/xhtml" prefix:a="A"></html>"#
        );
    }

    #[test]
    fn test_xhtml_namespace_without_prefix_dont_redeclare() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<prefix:html xmlns:prefix="https://www.w3.org/1999/xhtml"><prefix:body></prefix:body></prefix:html>"#)
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html xmlns="https://www.w3.org/1999/xhtml"><body></body></html>"#
        );
    }

    #[test]
    fn test_default_namespace_different_from_element_is_ignored_xhtml() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<prefix:html xmlns="different" xmlns:prefix="https://www.w3.org/1999/xhtml"><prefix:body></prefix:body></prefix:html>"#)
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html xmlns="https://www.w3.org/1999/xhtml"><body></body></html>"#
        );
    }

    #[test]
    fn test_default_namespace_different_from_element_is_ignored() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<prefix:html xmlns="different" xmlns:prefix="main"><prefix:body></prefix:body></prefix:html>"#)
            .unwrap();
        let s = xot.html5().to_string(root).unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><prefix:html xmlns:prefix="main"><prefix:body></prefix:body></prefix:html>"#
        );
    }

    // #[test]
    // fn test_xhtml_namespace_without_prefix_redeclare_if_intervening() {
    //     let mut xot = Xot::new();
    //     let root = xot
    //         .parse(r#"<prefix:html xmlns:prefix="https://www.w3.org/1999/xhtml"><prefix:body xmlns="different"><prefix:p></prefix:p></prefix:body></prefix:html>"#)
    //         .unwrap();
    //     let s = xot.html5().to_string(root).unwrap();
    //     // TODO: this is probably wrong; we don't expect an additional namespace declaration. On
    //     // the other hand, there was an intervening prefix, but it should have been ignored.
    //     assert_eq!(
    //         s,
    //         r#"<!DOCTYPE html><html xmlns="https://www.w3.org/1999/xhtml"><body></body></html>"#
    //     );
    // }

    #[test]
    fn test_pretty_with_xml_island() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<html><body><island xmlns="island"><foo><bar/></foo></island></body></html>"#)
            .unwrap();
        let s = xot
            .html5()
            .serialize_string(
                Parameters {
                    indentation: Some(Default::default()),
                    ..Default::default()
                },
                root,
            )
            .unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html>
  <body>
    <island xmlns="island">
      <foo>
        <bar></bar>
      </foo>
    </island>
  </body>
</html>
"#
        );
    }

    #[test]
    fn test_pretty_with_phrasing_element() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<html><body><p><span>Foo</span></p></body></html>"#)
            .unwrap();
        let s = xot
            .html5()
            .serialize_string(
                Parameters {
                    indentation: Some(Default::default()),
                    ..Default::default()
                },
                root,
            )
            .unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html>
  <body>
    <p><span>Foo</span></p>
  </body>
</html>
"#
        );
    }

    #[test]
    fn test_pretty_with_formatted_element() {
        let mut xot = Xot::new();
        // the HTML is nonsense, but we can verify p isn't indented as no
        // formatting may be removed or added inside of a formatted element like pre
        let root = xot
            .parse(r#"<html><body><pre><p></p></pre></body></html>"#)
            .unwrap();
        let s = xot
            .html5()
            .serialize_string(
                Parameters {
                    indentation: Some(Default::default()),
                    ..Default::default()
                },
                root,
            )
            .unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html>
  <body>
    <pre><p></p></pre>
  </body>
</html>
"#
        );
    }

    #[test]
    fn test_pretty_with_suppressed_element_exact_match() {
        let mut xot = Xot::new();
        let foo = xot.add_name("foo");
        let root = xot
            .parse(r#"<html><body><foo><p></p></foo></body></html>"#)
            .unwrap();
        let s = xot
            .html5()
            .serialize_string(
                Parameters {
                    indentation: Some(Indentation {
                        suppress: vec![foo],
                    }),
                    ..Default::default()
                },
                root,
            )
            .unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html>
  <body>
    <foo><p></p></foo>
  </body>
</html>
"#
        );
    }

    #[test]
    fn test_pretty_with_suppressed_element_case_insensitive_match() {
        let mut xot = Xot::new();
        let foo = xot.add_name("foo");
        let root = xot
            .parse(r#"<html><body><FOO><p></p></FOO></body></html>"#)
            .unwrap();
        let s = xot
            .html5()
            .serialize_string(
                Parameters {
                    indentation: Some(Indentation {
                        suppress: vec![foo],
                    }),
                    ..Default::default()
                },
                root,
            )
            .unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html>
  <body>
    <FOO><p></p></FOO>
  </body>
</html>
"#
        );
    }

    #[test]
    fn test_pretty_with_suppressed_element_case_no_case_insensitive_match_for_non_xhtml_namespace()
    {
        let mut xot = Xot::new();
        let foo = xot.add_name("foo");
        let root = xot
            .parse(
                r#"<html><body><prefix:FOO xmlns:prefix="ns"><p></p></prefix:FOO></body></html>"#,
            )
            .unwrap();
        let s = xot
            .html5()
            .serialize_string(
                Parameters {
                    indentation: Some(Indentation {
                        suppress: vec![foo],
                    }),
                    ..Default::default()
                },
                root,
            )
            .unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html>
  <body>
    <prefix:FOO xmlns:prefix="ns">
      <p></p>
    </prefix:FOO>
  </body>
</html>
"#
        );
    }

    #[test]
    fn test_pretty_with_suppressed_element_case_insensitive_match_xhtml_no_ns() {
        let mut xot = Xot::new();
        let foo = xot.add_name("foo");
        let root = xot
            .parse(&format!(
                r#"<html><body><FOO xmlns:xhtml="{}"><p></p></FOO></body></html>"#,
                XHTML_NS
            ))
            .unwrap();
        let s = xot
            .html5()
            .serialize_string(
                Parameters {
                    indentation: Some(Indentation {
                        suppress: vec![foo],
                    }),
                    ..Default::default()
                },
                root,
            )
            .unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html>
  <body>
    <FOO><p></p></FOO>
  </body>
</html>
"#
        );
    }

    #[test]
    fn test_pretty_with_suppressed_element_case_insensitive_match_no_ns_xhtml() {
        let mut xot = Xot::new();
        let xhtml_ns = xot.add_namespace(XHTML_NS);
        let foo = xot.add_name_ns("foo", xhtml_ns);
        let root = xot
            .parse(r#"<html><body><FOO><p></p></FOO></body></html>"#)
            .unwrap();
        let s = xot
            .html5()
            .serialize_string(
                Parameters {
                    indentation: Some(Indentation {
                        suppress: vec![foo],
                    }),
                    ..Default::default()
                },
                root,
            )
            .unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html>
  <body>
    <FOO><p></p></FOO>
  </body>
</html>
"#
        );
    }
}
