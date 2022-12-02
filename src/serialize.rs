use id_tree::Node;
use std::io::Error;
use std::io::Write;

use crate::xmlnode::{Document, XmlNode};

fn serialize(document: &Document, node: &Node<XmlNode>, w: &mut impl Write) -> Result<(), Error> {
    let xml_node = node.data();
    match xml_node {
        XmlNode::Element(element) => {}
        XmlNode::Text(text) => {}
    }
    todo!();
}
