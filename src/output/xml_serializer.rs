use std::io;

use crate::entity::{serialize_attribute, serialize_cdata, serialize_text};
use crate::error::Error;
use crate::id::NameId;
use crate::output::Normalizer;
use crate::xotdata::{Node, Xot};

use super::fullname::FullnameSerializer;
use super::{Output, OutputToken, Pretty, TokenSerializeParameters};

pub(crate) struct XmlSerializer<'a, N: Normalizer> {
    xot: &'a Xot,
    fullname_serializer: FullnameSerializer<'a>,
    normalizer: N,
    parameters: TokenSerializeParameters,
}

impl<'a, N: Normalizer> XmlSerializer<'a, N> {
    pub(crate) fn new(
        xot: &'a Xot,
        node: Node,
        parameters: TokenSerializeParameters,
        normalizer: N,
    ) -> Self {
        let extra_declarations = xot.namespaces_in_scope(node).collect();
        let fullname_serializer = FullnameSerializer::new(xot, extra_declarations);
        Self {
            xot,
            fullname_serializer,
            normalizer,
            parameters,
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
        let is_suppressed = |name_id| suppress.contains(&name_id);
        let mut pretty = Pretty::new(self.xot, is_suppressed, |_| false);
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
                OutputToken {
                    space: false,
                    text: format!(
                        "<{}",
                        self.fullname_serializer.element_fullname(element.name_id)?
                    ),
                }
            }
            StartTagClose => {
                if self.xot.first_child(node).is_none() {
                    OutputToken {
                        space: false,
                        text: "/>".to_string(),
                    }
                } else {
                    OutputToken {
                        space: false,
                        text: ">".to_string(),
                    }
                }
            }
            EndTag(element) => {
                let r = if self.xot.first_child(node).is_some() {
                    OutputToken {
                        space: false,
                        text: format!(
                            "</{}>",
                            self.fullname_serializer.element_fullname(element.name_id)?
                        ),
                    }
                } else {
                    OutputToken {
                        space: false,
                        text: "".to_string(),
                    }
                };
                self.fullname_serializer
                    .pop(self.xot.has_namespace_declarations(node));
                r
            }
            Prefix(prefix_id, namespace_id) => {
                // we don't want to output the xml prefix
                if *namespace_id == self.xot.xml_namespace() {
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
                // a text node can be a child of an element or document
                let parent = self.xot.parent(node).unwrap();

                let is_cdata_element = if let Some(element) = self.xot.element(parent) {
                    self.parameters
                        .cdata_section_elements
                        .contains(&element.name())
                } else {
                    false
                };

                if is_cdata_element {
                    OutputToken {
                        space: false,
                        text: serialize_cdata((*text).into(), &self.normalizer).to_string(),
                    }
                } else {
                    OutputToken {
                        space: false,
                        text: serialize_text(
                            (*text).into(),
                            &self.normalizer,
                            self.parameters.unescaped_gt,
                        )
                        .to_string(),
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
