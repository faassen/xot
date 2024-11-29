use crate::access::NodeEdge;
use crate::xmlvalue::{
    Attribute, Comment, Element, Namespace, ProcessingInstruction, Text, Value, ValueType,
};
use crate::xotdata::{Node, Xot};
use crate::NameId;

/// ## Value and type access
impl Xot {
    /// Access to the XML value for this node.
    ///
    /// ```rust
    /// use xot::{Xot, Value};
    ///
    /// let mut xot = Xot::new();
    ///
    /// let root = xot.parse("<doc>Example</doc>")?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let doc_name = xot.name("doc").unwrap();
    ///
    /// match xot.value(doc_el) {
    ///    Value::Element(element) => {
    ///       assert_eq!(element.name(), doc_name);
    ///   }
    ///   _ => { }
    /// }
    /// # Ok::<(), xot::Error>(())
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
    /// let root = xot.parse("<doc>Example</doc>")?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let text_node = xot.first_child(doc_el).unwrap();
    ///
    /// match xot.value_mut(doc_el) {
    ///    Value::Text(text) => {
    ///       text.set("Changed");
    ///   }
    ///   _ => { }
    /// }
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    ///
    /// Note that if you already know the type of a node value or are
    /// only interested in a single type, you can use the convenience
    /// methods like [`Xot::text_mut`]
    #[inline]
    pub fn value_mut(&mut self, node_id: Node) -> &mut Value {
        self.arena[node_id.get()].get_mut()
    }

    /// Get the [`ValueType`](crate::xmlvalue::ValueType) of a node.
    pub fn value_type(&self, node: Node) -> ValueType {
        self.value(node).value_type()
    }

    /// Return true if node is directly under the document node.
    /// This means it's either the document element or a comment or
    /// processing instruction.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<doc>Example</doc>")?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let text_node = xot.first_child(doc_el).unwrap();
    ///
    /// assert!(xot.has_document_parent(doc_el));
    /// assert!(!xot.has_document_parent(text_node));
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn has_document_parent(&self, node: Node) -> bool {
        if let Some(parent_id) = self.parent(node) {
            self.value_type(parent_id) == ValueType::Document
        } else {
            false
        }
    }

    /// Return true if the node is the document element.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<doc>Example</doc>")?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let text_node = xot.first_child(doc_el).unwrap();
    ///
    /// assert!(xot.is_document_element(doc_el));
    /// assert!(!xot.is_document_element(text_node));
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn is_document_element(&self, node: Node) -> bool {
        if let Some(parent_id) = self.parent(node) {
            self.value_type(parent_id) == ValueType::Document
                && self.value_type(node) == ValueType::Element
        } else {
            false
        }
    }

    /// Return true if node is the document node.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<doc>Example</doc>")?;
    /// let doc_el = xot.document_element(root).unwrap();
    ///
    /// assert!(xot.is_document(root));
    /// assert!(!xot.is_document(doc_el));
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn is_document(&self, node: Node) -> bool {
        self.value_type(node) == ValueType::Document
    }

    /// Return true if node is an element.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<doc>Example</doc>")?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let text_node = xot.first_child(doc_el).unwrap();
    ///
    /// assert!(xot.is_element(doc_el));
    /// assert!(!xot.is_element(text_node));
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn is_element(&self, node: Node) -> bool {
        self.value_type(node) == ValueType::Element
    }

    /// Return true if node is text.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<doc>Example</doc>")?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let text_node = xot.first_child(doc_el).unwrap();
    ///
    /// assert!(xot.is_text(text_node));
    /// assert!(!xot.is_text(doc_el));
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
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

    /// Return true if node is a namespace node.
    pub fn is_namespace_node(&self, node: Node) -> bool {
        self.value_type(node) == ValueType::Namespace
    }

    /// Return true if node is an attribute node.
    pub fn is_attribute_node(&self, node: Node) -> bool {
        self.value_type(node) == ValueType::Attribute
    }

    /// If this node's value is text, return a reference to it.
    ///
    /// Note that [`Xot::text_str()`] is a more convenient way to
    /// get the text value as a string slice.
    ///
    /// See also [`Xot::text_mut()`] if you want to manipulate
    /// a text value.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<doc>Example</doc>")?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let text_node = xot.first_child(doc_el).unwrap();
    ///
    /// assert_eq!(xot.text(text_node).unwrap().get(), "Example");
    /// assert!(xot.text(doc_el).is_none());
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn text(&self, node: Node) -> Option<&Text> {
        let xml_node = self.value(node);
        if let Value::Text(text) = xml_node {
            Some(text)
        } else {
            None
        }
    }

    /// If this node's value is text, return a reference to the string.
    ///
    /// ```rust
    ///
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<doc>Example</doc>")?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let text_node = xot.first_child(doc_el).unwrap();
    ///
    /// assert_eq!(xot.text_str(text_node).unwrap(), "Example");
    /// assert!(xot.text_str(doc_el).is_none());
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn text_str(&self, node: Node) -> Option<&str> {
        self.text(node).map(|n| n.get())
    }

    /// If this node's value is a text, return a mutable reference to it.
    ///
    /// This can be used to manipulate the text content of a
    /// document.
    ///
    /// ```rust
    ///
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<doc>Example</doc>")?;
    /// let doc_el = xot.document_element(root)?;
    /// let text_node = xot.first_child(doc_el).unwrap();
    ///
    /// let text = xot.text_mut(text_node).unwrap();
    ///
    /// text.set("New text");
    ///
    /// assert_eq!(xot.text_str(text_node).unwrap(), "New text");
    /// assert_eq!(xot.to_string(root)?, "<doc>New text</doc>");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn text_mut(&mut self, node: Node) -> Option<&mut Text> {
        let xml_node = self.value_mut(node);
        if let Value::Text(text) = xml_node {
            Some(text)
        } else {
            None
        }
    }

    /// Get the name of a node that's an element.
    ///
    /// If the node does not exist, then panic.
    pub fn get_element_name(&self, node: Node) -> NameId {
        match self.value(node) {
            Value::Element(element) => element.name(),
            _ => panic!("Node is not an element"),
        }
    }

    /// If this node's value is an element, return a reference to it.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<doc><child a="A"/></doc>"#)?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let child_el = xot.first_child(doc_el).unwrap();
    ///    
    /// let element = xot.element(child_el).unwrap();
    ///
    /// let child_name = xot.name("child").unwrap();
    /// assert_eq!(element.name(), child_name);
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn element(&self, node: Node) -> Option<&Element> {
        let xml_node = self.value(node);
        if let Value::Element(element) = xml_node {
            Some(element)
        } else {
            None
        }
    }

    /// If this node's value is an element, return a mutable reference to it.
    ///
    /// You can use this to change an element's name.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let changed = xot.add_name("changed");
    /// let root = xot.parse(r#"<doc><child/></doc>"#)?;
    /// let doc_el = xot.document_element(root)?;
    /// let child_el = xot.first_child(doc_el).unwrap();
    ///
    /// let element = xot.element_mut(child_el).unwrap();
    /// element.set_name(changed);
    ///
    /// assert_eq!(xot.to_string(root)?, r#"<doc><changed/></doc>"#);
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn element_mut(&mut self, node: Node) -> Option<&mut Element> {
        let xml_node = self.value_mut(node);
        if let Value::Element(element) = xml_node {
            Some(element)
        } else {
            None
        }
    }

    /// If this element has only a single text child, return a reference to it.
    ///
    /// If the element has no children or more than one child, return `None`.
    ///
    /// Note that [`Xot::text_content_str()`] is a more convenient way to get
    /// the text value as a string slice.
    ///
    /// See also [`Xot::text_content_mut()`] if you want to manipulate a text value.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<doc><a>Example</a><b/></doc>")?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let a_el = xot.first_child(doc_el).unwrap();
    /// let b_el = xot.next_sibling(a_el).unwrap();
    ///
    /// let text = xot.text_content(a_el).unwrap();
    ///
    /// assert_eq!(text.get(), "Example");
    /// assert!(xot.text_content(b_el).is_none());
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn text_content(&self, node: Node) -> Option<&Text> {
        if let Some(child) = self.first_child(node) {
            if self.next_sibling(child).is_some() {
                return None;
            }
            if let Some(text) = self.text(child) {
                return Some(text);
            }
        }
        None
    }

    /// If this element has only a single text child, return a mutable reference to it.
    ///
    /// If the element has no children, create a text child and return multiple reference
    /// to its value.
    ///
    /// If the element more than one child, return `None`.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<doc><a>Example</a><b/></doc>")?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let a_el = xot.first_child(doc_el).unwrap();
    /// let b_el = xot.next_sibling(a_el).unwrap();
    ///
    /// let text = xot.text_content_mut(a_el).unwrap();
    /// text.set("New value");
    ///
    /// assert_eq!(xot.to_string(root)?, "<doc><a>New value</a><b/></doc>");
    ///
    ///
    /// let text = xot.text_content_mut(b_el).unwrap();
    /// text.set("New value 2");
    ///
    /// assert_eq!(xot.to_string(root)?, "<doc><a>New value</a><b>New value 2</b></doc>");
    ///  
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn text_content_mut(&mut self, node: Node) -> Option<&mut Text> {
        if let Some(child) = self.first_child(node) {
            if self.next_sibling(child).is_some() {
                return None;
            }
            if let Some(text) = self.text_mut(child) {
                return Some(text);
            }
        } else if self.value_type(node) == ValueType::Element {
            self.append_text(node, "").unwrap();
            let child = self.first_child(node).unwrap();
            return self.text_mut(child);
        }
        None
    }

    /// If this element only has a single text child, return str reference to it.
    ///
    /// If the element has no content, return `Some("")`.
    ///
    /// If the element has more than one child, return `None`.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<doc><a>Example</a><b/><c><x/></c></doc>")?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let a_el = xot.first_child(doc_el).unwrap();
    /// let b_el = xot.next_sibling(a_el).unwrap();
    /// let c_el = xot.next_sibling(b_el).unwrap();
    ///
    /// let text = xot.text_content_str(a_el).unwrap();
    /// assert_eq!(text, "Example");
    /// let text = xot.text_content_str(b_el).unwrap();
    /// assert_eq!(text, "");
    /// assert!(xot.text_content_str(c_el).is_none());
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn text_content_str(&self, node: Node) -> Option<&str> {
        if self.first_child(node).is_none() {
            return Some("");
        }
        self.text_content(node).map(|n| n.get())
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

    /// Access namespace node value
    pub fn namespace_node(&self, node: Node) -> Option<&Namespace> {
        let xml_node = self.value(node);
        if let Value::Namespace(namespace) = xml_node {
            Some(namespace)
        } else {
            None
        }
    }

    /// Manipulate namespace node value
    pub fn namespace_node_mut(&mut self, node: Node) -> Option<&mut Namespace> {
        let xml_node = self.value_mut(node);
        if let Value::Namespace(namespace) = xml_node {
            Some(namespace)
        } else {
            None
        }
    }

    /// Access attribute node value
    pub fn attribute_node(&self, node: Node) -> Option<&Attribute> {
        let xml_node = self.value(node);
        if let Value::Attribute(attribute) = xml_node {
            Some(attribute)
        } else {
            None
        }
    }

    /// Manipulate attribute node value
    pub fn attribute_node_mut(&mut self, node: Node) -> Option<&mut Attribute> {
        let xml_node = self.value_mut(node);
        if let Value::Attribute(attribute) = xml_node {
            Some(attribute)
        } else {
            None
        }
    }

    /// Given a node, give back a string representation.
    ///
    /// For the root node and element nodes this gives back all text node
    /// descendant content, concatenated.
    ///
    /// For text nodes, it gives back the text.
    ///
    /// For comments, it gives back the comment text.
    ///
    /// For processing instructions, it gives back their content (data).
    ///
    /// For attribute nodes, it gives back the attribute value.
    ///
    /// For namespace nodes, it gives back the namespace URI.
    ///
    /// This is defined by the `string-value` property in
    /// <https://www.w3.org/TR/xpath-datamodel-31>
    pub fn string_value(&self, node: Node) -> String {
        match self.value(node) {
            Value::Document | Value::Element(_) => descendants_to_string(self, node),
            Value::Text(text) => text.get().to_string(),
            Value::ProcessingInstruction(pi) => pi.data().unwrap_or("").to_string(),
            Value::Comment(comment) => comment.get().to_string(),
            Value::Attribute(attribute) => attribute.value().to_string(),
            Value::Namespace(namespace) => {
                let namespace_id = namespace.namespace();
                self.namespace_str(namespace_id).to_string()
            }
        }
    }

    /// Check two nodes for semantic equality.
    ///
    /// This is a deep comparison of the nodes and their children.
    /// The trees have to have the same structure.
    ///
    /// A name is considered to be semantically equal to another name if
    /// they have the same namespace and local name. Prefixes are ignored.
    ///
    /// Two elements are the same if their name and attributes are the same. Namespace
    /// declarations are ignored.
    ///
    /// Text nodes, comments and processing instructions are considered to be the
    /// same if their values are the same.
    ///
    /// Compare two documents:
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root0 = xot.parse("<doc><a>Example</a><b/></doc>")?;
    /// let root1 = xot.parse("<doc><a>Example</a><b/></doc>")?;
    ///
    /// assert!(xot.deep_equal(root0, root1));
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    ///
    /// Different prefixes are ignored; the namespace URI is
    /// what is compared:
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root0 = xot.parse("<doc xmlns:foo='http://example.com'><foo:a/></doc>")?;
    /// let root1 = xot.parse("<doc xmlns:bar='http://example.com'><bar:a/></doc>")?;
    ///
    /// assert!(xot.deep_equal(root0, root1));
    /// # Ok::<(), xot::Error>(())
    /// ```
    ///
    /// But different text is a real difference:
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root0 = xot.parse("<doc>Example</doc>")?;
    /// let root1 = xot.parse("<doc>Changed</doc>")?;
    ///
    /// assert!(!xot.deep_equal(root0, root1));
    /// # Ok::<(), xot::Error>(())
    /// ```
    ///
    /// You can compare any nodes, not just documents:
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<doc><a f="F"/><b/><a f="F"/></doc>"#)?;
    /// let doc_el = xot.first_child(root).unwrap();
    /// let a_el = xot.first_child(doc_el).unwrap();
    /// let b_el = xot.next_sibling(a_el).unwrap();
    /// let a2_el = xot.next_sibling(b_el).unwrap();
    ///
    /// assert!(xot.deep_equal(a_el, a2_el));
    /// assert!(!xot.deep_equal(a_el, b_el));
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn deep_equal(&self, a: Node, b: Node) -> bool {
        self.advanced_deep_equal(a, b, |_| true, |a, b| a == b)
    }

    /// Compare the children of two nodes
    ///
    /// If the children are the same semantically, return true. It ignores
    /// the name and attributes of the `a` and `b` nodes themselves.
    pub fn deep_equal_children(&self, a: Node, b: Node) -> bool {
        let mut b_children = self.children(b);
        for a_child in self.children(a) {
            if let Some(b_child) = b_children.next() {
                // if the child is different, the element is different
                if !self.deep_equal(a_child, b_child) {
                    return false;
                }
            } else {
                // we cannot find a b child for an a child
                return false;
            }
        }
        b_children.next().is_none()
    }

    /// XPath deep equal
    /// Comparison of two nodes as defined by the XPath deep-equal function:
    ///
    /// <https://www.w3.org/TR/xpath-functions-31/#func-deep-equal>
    ///
    /// We ignore anything about typed content in that definition.
    pub fn deep_equal_xpath(
        &self,
        a: Node,
        b: Node,
        text_compare: impl Fn(&str, &str) -> bool,
    ) -> bool {
        // the top level comparison needs to compare the node, even if
        // processing instruction or a comment, though for elements, we want to
        // compare the structure and filter comments and processing
        // instructions out.
        use Value::*;
        match (self.value(a), self.value(b)) {
            (Element(_), Element(_)) | (Document, Document) => self.advanced_deep_equal(
                a,
                b,
                // we need to only consider elements and text nodes for
                // root/element content comparison
                |node| self.is_element(node) || self.is_text(node),
                text_compare,
            ),
            _ => self.advanced_compare_value(a, b, text_compare),
        }
    }

    /// Compare two nodes for semantic equality with custom text compare and
    /// filtering.
    ///
    /// This is a deep comparison of the nodes and their children. The trees
    /// have to have the same structure.
    ///
    /// A name is considered to be semantically equal to another name if they
    /// have the same namespace and local name. Prefixes are ignored.
    ///
    /// Two elements are the same if their name and attributes are the same.
    /// Namespace declarations are ignored.
    ///
    /// You can include only the nodes that are relevant to the comparison
    /// using the filter function.
    ///
    /// Text nodes and attributes are compared using the provided comparison function.
    pub fn advanced_deep_equal<F, C>(&self, a: Node, b: Node, filter: F, text_compare: C) -> bool
    where
        F: Fn(Node) -> bool,
        C: Fn(&str, &str) -> bool,
    {
        let filter_edge = |edge: &NodeEdge| {
            let node = match edge {
                NodeEdge::Start(node) | NodeEdge::End(node) => *node,
            };
            filter(node)
        };

        let mut edges_a = self.traverse(a).filter(filter_edge);
        let mut edges_b = self.traverse(b).filter(filter_edge);
        for edge_pair in edges_a.by_ref().zip(edges_b.by_ref()) {
            match edge_pair {
                (NodeEdge::Start(a), NodeEdge::Start(b)) => {
                    if !self.advanced_compare_value(a, b, &text_compare) {
                        return false;
                    }
                }
                (NodeEdge::End(_a), NodeEdge::End(_b)) => {
                    // If there is only a difference in structure, not value,
                    // the default case will fire
                }
                _ => {
                    return false;
                }
            }
        }
        // if we have leftover elements in the iterators, the trees are not equal
        if edges_a.next().is_some() || edges_b.next().is_some() {
            return false;
        }
        true
    }

    pub(crate) fn advanced_compare_value<C>(&self, a: Node, b: Node, text_compare: C) -> bool
    where
        C: Fn(&str, &str) -> bool,
    {
        let a_value = self.value(a);
        let b_value = self.value(b);
        match (a_value, b_value) {
            (Value::Document, Value::Document) => true,
            (Value::Element(a_element), Value::Element(b_element)) => {
                a_element.name() == b_element.name()
                    && self.advanced_compare_attributes(a, b, text_compare)
            }
            (Value::Text(a), Value::Text(b)) => text_compare(a.get(), b.get()),
            (Value::Comment(a), Value::Comment(b)) => a.get() == b.get(),
            (Value::ProcessingInstruction(a), Value::ProcessingInstruction(b)) => {
                if a.target() != b.target() {
                    return false;
                }
                match (a.data(), b.data()) {
                    (Some(a_content), Some(b_content)) => text_compare(a_content, b_content),
                    (None, None) => true,
                    _ => false,
                }
            }
            (Value::Attribute(a), Value::Attribute(b)) => {
                a.name() == b.name() && text_compare(a.value(), b.value())
            }
            (Value::Namespace(a), Value::Namespace(b)) => {
                a.prefix() == b.prefix() && a.namespace() == b.namespace()
            }
            _ => false,
        }
    }

    fn advanced_compare_attributes<C>(&self, a: Node, b: Node, text_compare: C) -> bool
    where
        C: Fn(&str, &str) -> bool,
    {
        let a_attributes = self.attributes(a);
        let b_attributes = self.attributes(b);
        if a_attributes.len() != b_attributes.len() {
            return false;
        }
        // if we can't find a value for a key in a in b, then we
        // know they aren't the same, given we already compared the length
        for (key, value_a) in a_attributes.iter() {
            let value_b = b_attributes.get(key);
            if let Some(value_b) = value_b {
                if !text_compare(value_a, value_b) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    /// Shallow compare two nodes.
    ///
    /// Does not consider content of any nodes, but does compare
    /// attributes.
    pub fn shallow_equal(&self, a: Node, b: Node) -> bool {
        self.shallow_equal_ignore_attributes(a, b, &[])
    }

    /// Shallow compare two nodes, with possibility to ignore attributes.
    ///
    /// Attributes of elements are compared, except those listed to ignore.
    ///
    /// Child content of the root or elements is not considered.
    pub fn shallow_equal_ignore_attributes(
        &self,
        a: Node,
        b: Node,
        ignore_attributes: &[NameId],
    ) -> bool {
        if let (Some(a_element), Some(b_element)) = (self.element(a), self.element(b)) {
            if a_element.name() != b_element.name() {
                return false;
            }

            // count the amount of attributes we compare
            let mut compare_attributes_count = 0;

            let a_attributes = self.attributes(a);
            let b_attributes = self.attributes(b);

            for (key, value_a) in a_attributes.iter() {
                if ignore_attributes.contains(&key) {
                    continue;
                }
                let value_b = b_attributes.get(key);
                if Some(value_a) != value_b {
                    return false;
                }
                compare_attributes_count += 1;
            }

            let mut b_ignore_attributes = 0;
            for ignore_attribute in ignore_attributes {
                if b_attributes.get(*ignore_attribute).is_some() {
                    b_ignore_attributes += 1;
                }
            }
            // we expect the amount of non-ignored attributes in a to
            // be the same as the amount of non-ignored attributes in b
            compare_attributes_count == b_attributes.len() - b_ignore_attributes
        } else {
            self.advanced_compare_value(a, b, |a, b| a == b)
        }
    }
}

fn descendants_to_string(xot: &Xot, node: Node) -> String {
    let texts = xot.descendants(node).filter_map(|n| xot.text_str(n));
    let (lower_bound, _) = texts.size_hint();
    let mut r = String::with_capacity(lower_bound);
    for text in texts {
        r.push_str(text);
    }
    r
}
