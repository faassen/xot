use crate::name::NameId;
use crate::xmldata::{Node, XmlData};
use crate::xmlvalue::{Comment, Element, ProcessingInstruction, Text, Value};

impl XmlData {
    pub(crate) fn new_node(&mut self, value: Value) -> Node {
        Node::new(self.arena.new_node(value))
    }

    pub fn new_text(&mut self, text: &str) -> Node {
        let text_node = Value::Text(Text::new(text.to_string()));
        self.new_node(text_node)
    }

    pub fn new_element(&mut self, name_id: NameId) -> Node {
        let element_node = Value::Element(Element::new(name_id));
        self.new_node(element_node)
    }

    pub fn new_comment(&mut self, comment: &str) -> Node {
        let comment_node = Value::Comment(Comment::new(comment.to_string()));
        self.new_node(comment_node)
    }

    pub fn new_processing_instruction(&mut self, target: &str, data: Option<&str>) -> Node {
        let pi_node = Value::ProcessingInstruction(ProcessingInstruction::new(
            target.to_string(),
            data.map(|s| s.to_string()),
        ));
        self.new_node(pi_node)
    }
}
