use id_tree::NodeId;
use std::io::Write;

use crate::error::Error;
use crate::xmlnode::{Document, XmlNode};

impl<'a> Document<'a> {
    pub fn serialize(
        self: &Document<'a>,
        node_id: &'a NodeId,
        w: &mut impl Write,
    ) -> Result<(), Error> {
        let xml_node = self.tree.get(node_id).unwrap().data();
        match xml_node {
            XmlNode::Element(element) => {
                let fullname = self.fullname(node_id, element.name_id)?;
                write!(w, "<{}", fullname)?;
                let mut children_ids = self.tree.children_ids(node_id)?.peekable();
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
