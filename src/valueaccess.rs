use crate::xmlvalue::{Comment, Element, ProcessingInstruction, Text, Value, ValueType};
use crate::xotdata::{Node, Xot};

/// ## Value and type access
impl<'a> Xot<'a> {
    /// Access to the XML value for this node.
    ///
    /// ```rust
    /// use xot::{Xot, Value};
    ///
    /// let mut xot = Xot::new();
    ///
    /// let root = xot.parse("<doc>Example</doc>").unwrap();
    /// let doc_el = xot.document_element(root).unwrap();
    /// let doc_name = xot.name("doc").unwrap();
    ///
    /// match xot.value(doc_el) {
    ///    Value::Element(element) => {
    ///       assert_eq!(element.name(), doc_name);
    ///   }
    ///   _ => { }
    /// }
    /// ```
    ///
    /// Note that if you already know the type of a node value or are
    /// only interested in a single type, you can use the convenience
    /// methods like [`Xot::element`].
    #[inline]
    pub fn value(&self, node_id: Node) -> &Value {
        self.arena[node_id.get()].get()
    }

    /// Mutable access to the XML value for this node.
    ///
    /// ```rust
    /// use xot::{Xot, Value};
    ///
    /// let mut xot = Xot::new();
    ///
    /// let root = xot.parse("<doc>Example</doc>").unwrap();
    /// let doc_el = xot.document_element(root).unwrap();
    ///
    /// let attr_name = xot.add_name("foo");
    ///
    /// match xot.value_mut(doc_el) {
    ///    Value::Element(element) => {
    ///       element.set_attribute(attr_name, "Foo!")
    ///   }
    ///   _ => { }
    /// }
    ///
    /// ```
    /// Note that if you already know the type of a node value or are
    /// only interested in a single type, you can use the convenience
    /// methods like [`Xot::element_mut`]
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
