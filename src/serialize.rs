use std::io::Write;

use crate::document::Document;
use crate::entity::serialize_text;
use crate::error::Error;
use crate::name::NameId;
use crate::xmldata::{Node, NodeEdge, XmlData};
use crate::xmlvalue::{ToPrefix, Value};

impl Document {
    pub fn serialize_node(
        &self,
        node_id: Node,
        w: &mut impl Write,
        data: &XmlData,
    ) -> Result<(), Error> {
        let mut fullname_serializer = FullnameSerializer::new(data);
        for edge in data.traverse(node_id) {
            match edge {
                NodeEdge::Start(node_id) => {
                    self.handle_edge_start(node_id, w, &mut fullname_serializer, data)?;
                }
                NodeEdge::End(node_id) => {
                    self.handle_edge_end(node_id, w, &mut fullname_serializer, data)?;
                }
            }
        }
        Ok(())
    }

    pub fn serialize_to_string(&self, data: &XmlData) -> Result<String, Error> {
        let mut buf = Vec::new();
        self.serialize_node(self.root(), &mut buf, data)?;
        Ok(String::from_utf8(buf).unwrap())
    }

    fn handle_edge_start(
        &self,
        node_id: Node,
        w: &mut impl Write,
        fullname_serializer: &mut FullnameSerializer,
        data: &XmlData,
    ) -> Result<(), Error> {
        let node = &data.arena[node_id.get()];
        let xml_node = node.get();
        match xml_node {
            Value::Root => {}
            Value::Element(element) => {
                if !element.namespace_info.to_prefix.is_empty() {
                    fullname_serializer.push(&element.namespace_info.to_prefix);
                }
                let fullname = fullname_serializer.fullname(element.name_id)?;
                write!(w, "<{}", fullname)?;
                for (prefix_id, namespace_id) in element.namespace_info.to_namespace.iter() {
                    let namespace = data.namespace_lookup.get_value(*namespace_id);
                    if prefix_id == &data.empty_prefix_id {
                        write!(w, " xmlns=\"{}\"", namespace)?;
                    } else {
                        write!(
                            w,
                            " xmlns:{}=\"{}\"",
                            data.prefix_lookup.get_value(*prefix_id),
                            namespace
                        )?;
                    }
                }
                for (name_id, value) in element.attributes.iter() {
                    let fullname = fullname_serializer.fullname(*name_id)?;
                    write!(w, " {}=\"{}\"", fullname, serialize_text(value.into()))?;
                }

                if node.first_child().is_none() {
                    write!(w, "/>")?;
                } else {
                    write!(w, ">")?;
                }
            }
            Value::Text(text) => {
                write!(w, "{}", serialize_text(text.get().into()))?;
            }
            Value::Comment(comment) => {
                write!(w, "<!--{}-->", comment.get())?;
            }
            Value::ProcessingInstruction(pi) => {
                if let Some(data) = pi.get_data() {
                    write!(w, "<?{} {}?>", pi.get_target(), data)?;
                } else {
                    write!(w, "<?{}?>", pi.get_target())?;
                }
            }
        }
        Ok(())
    }

    fn handle_edge_end(
        &self,
        node_id: Node,
        w: &mut impl Write,
        fullname_serializer: &mut FullnameSerializer,
        data: &XmlData,
    ) -> Result<(), Error> {
        let node = &data.arena[node_id.get()];
        let xml_node = node.get();
        if let Value::Element(element) = xml_node {
            if node.first_child().is_some() {
                let fullname = fullname_serializer.fullname(element.name_id)?;
                write!(w, "</{}>", fullname)?;
            }
            if !element.namespace_info.to_prefix.is_empty() {
                fullname_serializer.pop();
            }
        }
        Ok(())
    }
}

struct FullnameSerializer<'a> {
    data: &'a XmlData,
    prefix_stack: Vec<ToPrefix>,
}

impl<'a> FullnameSerializer<'a> {
    fn new(data: &'a XmlData) -> Self {
        Self {
            data,
            prefix_stack: Vec::new(),
        }
    }

    fn push(&mut self, to_prefix: &ToPrefix) {
        let entry = if self.prefix_stack.is_empty() {
            to_prefix.clone()
        } else {
            let mut entry = self.top().clone();
            entry.extend(to_prefix);
            entry
        };
        self.prefix_stack.push(entry);
    }

    fn pop(&mut self) {
        self.prefix_stack.pop();
    }

    #[inline]
    fn top(&self) -> &ToPrefix {
        &self.prefix_stack[self.prefix_stack.len() - 1]
    }

    fn fullname(&self, name_id: NameId) -> Result<String, Error> {
        let name = self.data.name_lookup.get_value(name_id);
        if name.namespace_id == self.data.no_namespace_id {
            return Ok(name.name.to_string());
        }
        let prefix_id = if !self.prefix_stack.is_empty() {
            self.top().get(&name.namespace_id)
        } else {
            None
        };
        // if prefix_id cannot be found, then that's an error: we have removed
        // a prefix declaration even though it is still in use
        let prefix_id = *prefix_id.ok_or_else(|| {
            Error::NoPrefixForNamespace(
                self.data
                    .namespace_lookup
                    .get_value(name.namespace_id)
                    .to_string(),
            )
        })?;
        if prefix_id == self.data.empty_prefix_id {
            Ok(name.name.to_string())
        } else {
            let prefix = self.data.prefix_lookup.get_value(prefix_id);
            Ok(format!("{}:{}", prefix, name.name))
        }
    }
}
