use indextree::{NodeEdge, NodeId};
use std::io::Write;

use crate::document::Document;
use crate::error::Error;
use crate::xmlnode::XmlNode;

impl<'a> Document<'a> {
    pub fn serialize(
        self: &Document<'a>,
        node_id: NodeId,
        w: &mut impl Write,
    ) -> Result<(), Error> {
        for edge in node_id.traverse(&self.data.arena) {
            match edge {
                NodeEdge::Start(node_id) => {
                    self.handle_edge_start(node_id, w)?;
                }
                NodeEdge::End(node_id) => {
                    self.handle_edge_end(node_id, w)?;
                }
            }
        }
        Ok(())
    }

    fn handle_edge_start(&self, node_id: NodeId, w: &mut impl Write) -> Result<(), Error> {
        let xml_node = self.data.arena.get(node_id).unwrap().get();
        match xml_node {
            XmlNode::Root => {}
            XmlNode::Element(element) => {
                let fullname = self.fullname(node_id, element.name_id)?;
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

    fn handle_edge_end(&self, node_id: NodeId, w: &mut impl Write) -> Result<(), Error> {
        let xml_node = self.data.arena.get(node_id).unwrap().get();
        match xml_node {
            XmlNode::Root => {}
            XmlNode::Element(element) => {
                if node_id.children(&self.data.arena).next().is_some() {
                    let fullname = self.fullname(node_id, element.name_id)?;
                    write!(w, "</{}>", fullname)?;
                }
            }
            XmlNode::Text(text) => {}
        }
        Ok(())
    }
}
