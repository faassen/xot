use indextree::NodeId;
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
        let xml_node = self.arena.get(node_id).unwrap().get();
        match xml_node {
            XmlNode::Root => {
                for child in node_id.children(self.arena) {
                    self.serialize(child, w)?;
                }
            }
            XmlNode::Element(element) => {
                let fullname = self.fullname(node_id, element.name_id)?;
                write!(w, "<{}", fullname)?;
                let mut children_ids = node_id.children(&self.arena).peekable();
                if children_ids.peek().is_none() {
                    write!(w, "/>")?;
                } else {
                    write!(w, ">")?;
                    for child_id in children_ids {
                        self.serialize(child_id, w)?;
                    }
                    write!(w, "</{}>", fullname)?;
                }
            }
            XmlNode::Text(text) => {
                write!(w, "{}", text)?;
            }
        }
        Ok(())
    }
}
