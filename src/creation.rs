use crate::name::NameId;
use crate::xmlvalue::{Comment, Element, ProcessingInstruction, Text, Value};
use crate::xotdata::{Node, Xot};

/// ## Creation
/// See also the convenience manipulation methods like [`Xot::append_element`]
/// in the manipulation section.
impl<'a> Xot<'a> {
    pub(crate) fn new_node(&mut self, value: Value) -> Node {
        Node::new(self.arena.new_node(value))
    }

    /// Create a new, unattached root node.
    ///
    /// You can use this to create a new document from scratch.
    pub fn new_root(&mut self) -> Node {
        let root = Value::Root;
        self.new_node(root)
    }

    /// Create a new, unattached element node given element name.
    ///
    /// You supply a name.
    ///  
    /// To create a potentially new name you can use [`Xot::add_name`] or
    /// [`Xot::add_name_ns`]. If the name already exists
    /// the existing name id is returned.
    ///
    /// To reuse an existing name that has been
    /// previously used, you can use
    /// [`Xot::name`] or [`Xot::name_ns`].
    pub fn new_element(&mut self, name: NameId) -> Node {
        let element = Value::Element(Element::new(name));
        self.new_node(element)
    }

    /// Create a new, unattached text node.
    pub fn new_text(&mut self, text: &str) -> Node {
        let text = Value::Text(Text::new(text.to_string()));
        self.new_node(text)
    }

    /// Create a new, unattached comment node given comment text.
    pub fn new_comment(&mut self, comment: &str) -> Node {
        let comment = Value::Comment(Comment::new(comment.to_string()));
        self.new_node(comment)
    }

    /// Create a new, unattached processing instruction.
    pub fn new_processing_instruction(&mut self, target: &str, data: Option<&str>) -> Node {
        let pi = Value::ProcessingInstruction(ProcessingInstruction::new(
            target.to_string(),
            data.map(|s| s.to_string()),
        ));
        self.new_node(pi)
    }
}
