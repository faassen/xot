use std::io;

use ahash::{HashSet, HashSetExt};

use crate::entity::{
    serialize_attribute, serialize_cdata, serialize_text, serialize_text_no_escape,
};
use crate::error::Error;
use crate::fullname::FullnameSerializer;
use crate::id::NameId;
use crate::output::Normalizer;
use crate::xotdata::{Node, Xot};
use crate::NamespaceId;

use super::serializer::get_extra_prefixes;
use super::{Output, OutputToken, Pretty};

// used to determine whether something is a HTML 5 element
const XHTML_NS: &str = "https://www.w3.org/1999/xhtml";

#[derive(Debug)]
pub(crate) struct Html5Elements {
    xhtml_namespace_id: NamespaceId,
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
            void_names,
            phrasing_content_names,
            formatted_names,
            no_escape_names,
        }
    }

    fn is_html_element(&self, xot: &Xot, name_id: NameId) -> bool {
        let namespace = xot.namespace_for_name(name_id);
        namespace == self.xhtml_namespace_id || namespace == xot.no_namespace()
    }
}

pub(crate) struct Html5Serializer<'a, N: Normalizer> {
    xot: &'a Xot,
    html5_elements: &'a Html5Elements,
    cdata_section_names: &'a [NameId],
    fullname_serializer: FullnameSerializer<'a>,
    normalizer: N,
}

impl<'a, N: Normalizer> Html5Serializer<'a, N> {
    pub(crate) fn new(
        xot: &'a Xot,
        html5_elements: &'a Html5Elements,
        node: Node,
        cdata_section_names: &'a [NameId],
        normalizer: N,
    ) -> Self {
        let extra_prefixes = get_extra_prefixes(xot, node);
        let mut fullname_serializer = FullnameSerializer::new(xot);
        fullname_serializer.push(&extra_prefixes);
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
        let mut pretty = Pretty::new(self.xot, suppress);
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
                self.fullname_serializer.push(&self.xot.prefixes(node));
                OutputToken {
                    space: false,
                    text: format!(
                        "<{}",
                        self.fullname_serializer.fullname_or_err(element.name_id)?
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
                            self.fullname_serializer.fullname_or_err(element.name_id)?
                        ),
                    }
                };
                self.fullname_serializer.pop();
                r
            }
            Prefix(prefix_id, namespace_id) => {
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
                let fullname = self.fullname_serializer.fullname_attr_or_err(*name_id)?;
                OutputToken {
                    space: true,
                    text: format!(
                        "{}=\"{}\"",
                        fullname,
                        serialize_attribute((*value).into(), &self.normalizer)
                    ),
                }
            }
            Text(text) => {
                // a text node is always a child of an element
                let parent = self.xot.parent(node).unwrap();
                let element = self.xot.element(parent).unwrap();
                if self
                    .html5_elements
                    .no_escape_names
                    .matches(self.xot, element.name())
                {
                    OutputToken {
                        space: false,
                        text: serialize_text_no_escape((*text).into(), &self.normalizer)
                            .to_string(),
                    }
                } else if self.cdata_section_names.contains(&element.name()) {
                    OutputToken {
                        space: false,
                        text: serialize_cdata((*text).into(), &self.normalizer).to_string(),
                    }
                } else {
                    OutputToken {
                        space: false,
                        text: serialize_text((*text).into(), &self.normalizer).to_string(),
                    }
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
                if let Some(data) = data {
                    OutputToken {
                        space: false,
                        text: format!("<?{} {}?>", target, data),
                    }
                } else {
                    OutputToken {
                        space: false,
                        text: format!("<?{}?>", target),
                    }
                }
            }
        };
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
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
}
