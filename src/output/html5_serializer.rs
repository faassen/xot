use std::io;

use ahash::{HashSet, HashSetExt};

use crate::entity::{serialize_attribute, serialize_cdata, serialize_text};
use crate::error::Error;
use crate::fullname::FullnameSerializer;
use crate::id::{Name, NameId, NameLookup, NamespaceLookup};
use crate::output::Normalizer;
use crate::xotdata::{Node, Xot};
use crate::NamespaceId;

use super::serializer::get_extra_prefixes;
use super::{Output, OutputToken, Pretty};

// used to determine whether something is a HTML 5 element
const XHTML_NS: &str = "https://www.w3.org/1999/xhtml";

#[derive(Debug)]
pub(crate) struct Html5Elements {
    xhtml_ns: NamespaceId,
    name_ids: HashSet<NameId>,
    names: HashSet<String>,
}

impl Html5Elements {
    pub(crate) fn new(
        namespace_lookup: &mut NamespaceLookup,
        name_lookup: &mut NameLookup,
        no_namespace_id: NamespaceId,
    ) -> Self {
        let names = [
            "area", "base", "br", "col", "embed", "hr", "img", "input", "keygen", "link", "meta",
            "param", "source", "track", "wbr",
            // extra elements not in the HTML5 spec but null in HTML 4
            "basefont", "frame", "isindex",
        ];
        let mut name_ids = HashSet::new();
        let xhtml_ns = namespace_lookup.get_id_mut(XHTML_NS);
        for name in names {
            // lowercase names, no namespace
            name_ids.insert(name_lookup.get_id_mut(&Name::new(name, no_namespace_id)));
            // uppercase names, no namespace
            name_ids.insert(
                name_lookup.get_id_mut(&Name::new(&name.to_ascii_uppercase(), no_namespace_id)),
            );
            // lowercase names, XHTML namespace
            name_ids.insert(name_lookup.get_id_mut(&Name::new(name, xhtml_ns)));
            // uppercase names, XHTML namespace
            name_ids
                .insert(name_lookup.get_id_mut(&Name::new(&name.to_ascii_uppercase(), xhtml_ns)));
        }
        Self {
            xhtml_ns,
            names: names.iter().map(|name| name.to_string()).collect(),
            name_ids,
        }
    }

    fn is_html_element(&self, xot: &Xot, name_id: NameId) -> bool {
        let namespace = xot.namespace_for_name(name_id);
        namespace == self.xhtml_ns || namespace == xot.no_namespace()
    }

    fn is_void(&self, xot: &Xot, name_id: NameId) -> bool {
        if self.name_ids.contains(&name_id) {
            return true;
        }
        if !self.is_html_element(xot, name_id) {
            return false;
        }
        let name = xot.local_name_str(name_id);
        // now lowercase the name and look it up
        let name = name.to_ascii_lowercase();
        self.names.contains(&name)
    }
}

pub(crate) struct Html5Serializer<'a, N: Normalizer> {
    xot: &'a Xot,
    cdata_section_names: &'a [NameId],
    fullname_serializer: FullnameSerializer<'a>,
    normalizer: N,
}

impl<'a, N: Normalizer> Html5Serializer<'a, N> {
    pub(crate) fn new(
        xot: &'a Xot,
        node: Node,
        cdata_section_names: &'a [NameId],
        normalizer: N,
    ) -> Self {
        let extra_prefixes = get_extra_prefixes(xot, node);
        let mut fullname_serializer = FullnameSerializer::new(xot);
        fullname_serializer.push(&extra_prefixes);
        Self {
            xot,
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
                let r = if self.xot.html5_elements.is_void(self.xot, element.name()) {
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
                if self.cdata_section_names.contains(&element.name()) {
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
        let s = xot.to_html5_string(root).unwrap();
        assert_eq!(s, "<!DOCTYPE html><html><head></head><body></body></html>");
    }

    #[test]
    fn test_never_empty_xml_element() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<html><head><foo xmlns="foo"><bar></bar></foo></head><body></body></html>"#)
            .unwrap();
        let s = xot.to_html5_string(root).unwrap();
        assert_eq!(
            s,
            r#"<!DOCTYPE html><html><head><foo xmlns="foo"><bar></bar></foo></head><body></body></html>"#
        );
    }

    #[test]
    fn test_void_element() {
        let mut xot = Xot::new();
        let root = xot.parse("<html><body>foo<br/>bar</body></html>").unwrap();
        let s = xot.to_html5_string(root).unwrap();
        assert_eq!(s, "<!DOCTYPE html><html><body>foo<br>bar</body></html>");
    }
}
