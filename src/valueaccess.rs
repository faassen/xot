use crate::xmlvalue::{Comment, Element, ProcessingInstruction, Text, Value, ValueType};
use crate::xotdata::{Node, Xot};

/// Obtain XML values and their types.
///
/// These are handy if you only need to match against a single value or know
/// the value type already. If you want to handle all value types, use a
/// `match` statement on [`Value`](crate::xmlvalue::Value) instead.
impl Xot {
    /// Access to the XML value for this node.
    ///
    /// ```rust
    /// use xot::{Xot, Value};
    ///
    /// let mut xot = Xot::new();
    ///
    /// let doc = xot.parse("<doc>Example</doc>").unwrap();
    /// let root = xot.document_element(doc).unwrap();
    /// let doc_name = xot.name("doc").unwrap();
    ///
    /// match xot.value(root) {
    ///    Value::Element(element) => {
    ///       assert_eq!(element.name_id(), doc_name);
    ///   }
    ///   _ => { }
    /// }
    /// ```
    #[inline]
    pub fn value(&self, node_id: Node) -> &Value {
        self.arena[node_id.get()].get()
    }

    /// Mutable access to the XML value for this node.
    #[inline]
    pub fn value_mut(&mut self, node_id: Node) -> &mut Value {
        self.arena[node_id.get()].get_mut()
    }

    /// Get the [`ValueType`](crate::xmlvalue::ValueType) of a node.
    pub fn value_type(&self, node: Node) -> ValueType {
        self.value(node).value_type()
    }

    /// Return true if node is directly under the document root.
    /// This means it's either the document element or a comment or
    /// processing instruction.
    pub fn is_under_root(&self, node: Node) -> bool {
        if let Some(parent_id) = self.parent(node) {
            self.value_type(parent_id) == ValueType::Root
        } else {
            false
        }
    }

    /// Return true if node is the document root.
    pub fn is_root(&self, node: Node) -> bool {
        self.value_type(node) == ValueType::Root
    }

    /// Return true if node is an element.
    pub fn is_element(&self, node: Node) -> bool {
        self.value_type(node) == ValueType::Element
    }

    /// Return true if node is text.
    pub fn is_text(&self, node: Node) -> bool {
        self.value_type(node) == ValueType::Text
    }

    /// Return true if node is a comment.
    pub fn is_comment(&self, node: Node) -> bool {
        self.value_type(node) == ValueType::Comment
    }

    /// Return true if node is a processing instruction.
    pub fn is_processing_instruction(&self, node: Node) -> bool {
        self.value_type(node) == ValueType::ProcessingInstruction
    }

    /// If this node's value is text, return a reference to it.
    pub fn text(&self, node: Node) -> Option<&Text> {
        let xml_node = self.value(node);
        if let Value::Text(text) = xml_node {
            Some(text)
        } else {
            None
        }
    }

    /// If this node's value is text, return a reference to the string.    
    pub fn text_str(&self, node: Node) -> Option<&str> {
        self.text(node).map(|n| n.get())
    }

    /// If this node's value is a text, return a mutable reference to it.
    pub fn text_mut(&mut self, node: Node) -> Option<&mut Text> {
        let xml_node = self.value_mut(node);
        if let Value::Text(text) = xml_node {
            Some(text)
        } else {
            None
        }
    }

    /// If this node's value is an element, return a reference to it.
    pub fn element(&self, node: Node) -> Option<&Element> {
        let xml_node = self.value(node);
        if let Value::Element(element) = xml_node {
            Some(element)
        } else {
            None
        }
    }

    /// If this node's value is an element, return a mutable reference to it.
    pub fn element_mut(&mut self, node: Node) -> Option<&mut Element> {
        let xml_node = self.value_mut(node);
        if let Value::Element(element) = xml_node {
            Some(element)
        } else {
            None
        }
    }

    /// If this node's value is a comment, return a reference to it.
    pub fn comment(&self, node: Node) -> Option<&Comment> {
        let xml_node = self.value(node);
        if let Value::Comment(comment) = xml_node {
            Some(comment)
        } else {
            None
        }
    }

    /// If this node's value is a comment, return a reference to the string.
    pub fn comment_str(&self, node: Node) -> Option<&str> {
        self.comment(node).map(|n| n.get())
    }

    /// If this node's value is a comment, return a mutable reference to it.
    pub fn comment_mut(&mut self, node: Node) -> Option<&mut Comment> {
        let xml_node = self.value_mut(node);
        if let Value::Comment(comment) = xml_node {
            Some(comment)
        } else {
            None
        }
    }

    /// If this node's value is a processing instruction, return a reference to it.
    pub fn processing_instruction(&self, node: Node) -> Option<&ProcessingInstruction> {
        let xml_node = self.value(node);
        if let Value::ProcessingInstruction(pi) = xml_node {
            Some(pi)
        } else {
            None
        }
    }

    /// If this node's value is a processing instruction, return a mutable reference to it.
    pub fn processing_instruction_mut(&mut self, node: Node) -> Option<&mut ProcessingInstruction> {
        let xml_node = self.value_mut(node);
        if let Value::ProcessingInstruction(pi) = xml_node {
            Some(pi)
        } else {
            None
        }
    }
}
