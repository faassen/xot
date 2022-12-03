use indextree::{NodeEdge, NodeId};
use std::io::Write;
use vector_map::VecMap;

use crate::document::{Document, XmlData};
use crate::error::Error;
use crate::name::NameId;
use crate::namespace::NamespaceId;
use crate::prefix::PrefixId;
use crate::xmlnode::{ToPrefix, XmlNode};

impl<'a> Document<'a> {
    pub fn serialize(
        self: &Document<'a>,
        node_id: NodeId,
        w: &mut impl Write,
    ) -> Result<(), Error> {
        let mut fullname_serializer = FullnameSerializer::new(self.data);
        for edge in node_id.traverse(&self.data.arena) {
            match edge {
                NodeEdge::Start(node_id) => {
                    self.handle_edge_start(node_id, w, &mut fullname_serializer)?;
                }
                NodeEdge::End(node_id) => {
                    self.handle_edge_end(node_id, w, &mut fullname_serializer)?;
                }
            }
        }
        Ok(())
    }

    fn handle_edge_start(
        &self,
        node_id: NodeId,
        w: &mut impl Write,
        fullname_serializer: &mut FullnameSerializer,
    ) -> Result<(), Error> {
        let xml_node = self.data.arena.get(node_id).unwrap().get();
        match xml_node {
            XmlNode::Root => {}
            XmlNode::Element(element) => {
                if !element.namespace_info.to_prefix.is_empty() {
                    fullname_serializer.push(&element.namespace_info.to_prefix);
                }
                let fullname = fullname_serializer.fullname(element.name_id)?;
                write!(w, "<{}", fullname)?;
                for (prefix_id, namespace_id) in element.namespace_info.to_namespace.iter() {
                    let namespace = self.data.namespace_lookup.get_value(*namespace_id);
                    if prefix_id == &self.data.empty_prefix_id {
                        write!(w, " xmlns=\"{}\"", namespace)?;
                    } else {
                        write!(
                            w,
                            " xmlns:{}=\"{}\"",
                            self.data.prefix_lookup.get_value(*prefix_id),
                            namespace
                        )?;
                    }
                }
                if node_id.children(&self.data.arena).next().is_none() {
                    write!(w, "/>")?;
                } else {
                    write!(w, ">")?;
                }
            }
            XmlNode::Text(text) => {
                write!(w, "{}", text)?;
            }
        }
        Ok(())
    }

    fn handle_edge_end(
        &self,
        node_id: NodeId,
        w: &mut impl Write,
        fullname_serializer: &mut FullnameSerializer,
    ) -> Result<(), Error> {
        let xml_node = self.data.arena.get(node_id).unwrap().get();
        match xml_node {
            XmlNode::Root => {}
            XmlNode::Element(element) => {
                if node_id.children(&self.data.arena).next().is_some() {
                    let fullname = fullname_serializer.fullname(element.name_id)?;
                    write!(w, "</{}>", fullname)?;
                }
                if !element.namespace_info.to_prefix.is_empty() {
                    fullname_serializer.pop();
                }
            }
            XmlNode::Text(text) => {}
        }
        Ok(())
    }
}

struct FullnameSerializer<'a> {
    data: &'a XmlData<'a>,
    prefix_stack: Vec<ToPrefix>,
}

impl<'a> FullnameSerializer<'a> {
    fn new(data: &'a XmlData<'a>) -> Self {
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
            Ok(format!("{}", name.name))
        } else {
            let prefix = self.data.prefix_lookup.get_value(prefix_id);
            Ok(format!("{}:{}", prefix, name.name))
        }
    }
}
