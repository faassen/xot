use crate::name::NameId;
use crate::xmldata::{Node, XmlData};
use crate::xmlvalue::{Comment, Element, ProcessingInstruction, Text, Value};

/// XML node creation.
impl XmlData {
    pub(crate) fn new_node(&mut self, value: Value) -> Node {
        Node::new(self.arena.new_node(value))
    }

    /// Create a new, unattached text node.
    pub fn new_text(&mut self, text: &str) -> Node {
        let text_node = Value::Text(Text::new(text.to_string()));
        self.new_node(text_node)
    }

    /// Create a new, unattached element node given element name.
    ///
    /// You supply a name id.
    ///  
    /// To create a potentially new name id you can use [`XmlData::add_name`] or
    /// [`XmlData::add_name_ns`]. If the name already exists
    /// the existing name id is returned.
    ///
    /// To reuse an existing name that has been
    /// previously used, you can use
    /// [`XmlData::name`] or [`XmlData::name_ns`].
    pub fn new_element(&mut self, name_id: NameId) -> Node {
        let element_node = Value::Element(Element::new(name_id));
        self.new_node(element_node)
    }

    /// Create a new, unattached comment node given comment text.
    pub fn new_comment(&mut self, comment: &str) -> Node {
        let comment_node = Value::Comment(Comment::new(comment.to_string()));
        self.new_node(comment_node)
    }

    /// Create a new, unattached processing instruction.
    pub fn new_processing_instruction(&mut self, target: &str, data: Option<&str>) -> Node {
        let pi_node = Value::ProcessingInstruction(ProcessingInstruction::new(
            target.to_string(),
            data.map(|s| s.to_string()),
        ));
        self.new_node(pi_node)
    }
}
