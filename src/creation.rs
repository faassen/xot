use crate::error::Error;
use crate::name::NameId;
use crate::xmlvalue::{Comment, Element, FullValue, ProcessingInstruction, Text, Value};
use crate::xotdata::{Node, Xot};

/// ## Creation
/// See also the convenience manipulation methods like [`Xot::append_element`]
/// in the manipulation section.
impl Xot {
    pub(crate) fn new_node(&mut self, value: Value) -> Node {
        Node::new(self.arena.new_node(FullValue::Value(value)))
    }

    /// Create a new root node.
    ///
    /// You can use this to create a new document from scratch.
    /// You have to supply a document element, as a root without
    /// a document element is not allowed in XML. If you want to do
    /// this manually, use `Xot::new_root_unconnected`.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let doc_name = xot.add_name("doc");
    /// let doc_el = xot.new_element(doc_name);
    /// let txt = xot.new_text("Hello, world!");
    /// xot.append(doc_el, txt)?;
    ///
    /// /// now create the root
    /// let root = xot.new_root(doc_el)?;
    ///
    /// assert_eq!(xot.to_string(root)?, "<doc>Hello, world!</doc>");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn new_root(&mut self, node: Node) -> Result<Node, Error> {
        if !self.is_element(node) {
            return Err(Error::InvalidOperation(
                "You must supply an element node".to_string(),
            ));
        }
        let root_node = self.new_root_unconnected();
        self.append(root_node, node)?;
        Ok(root_node)
    }

    /// Create a new, unattached root node without document element.
    ///
    /// You can use this to create a new document from scratch.
    /// If you don't attach at a single element later, the document
    /// is going to be invalid.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let doc_name = xot.add_name("doc");
    /// let doc_el = xot.new_element(doc_name);
    /// let txt = xot.new_text("Hello, world!");
    /// xot.append(doc_el, txt)?;
    ///
    /// /// now create the root
    /// let root = xot.new_root_unconnected();
    /// xot.append(root, doc_el)?;
    ///
    /// assert_eq!(xot.to_string(root)?, "<doc>Hello, world!</doc>");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn new_root_unconnected(&mut self) -> Node {
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
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let doc_name = xot.add_name("doc");
    /// let doc_el = xot.new_element(doc_name);
    ///
    /// let root = xot.new_root(doc_el)?;
    /// assert_eq!(xot.to_string(root)?, "<doc/>");
    /// # Ok::<(), xot::Error>(())
    /// ```
    ///
    /// With a namespaced element:
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let ns = xot.add_namespace("http://example.com");
    /// let ex = xot.add_prefix("ex");
    ///
    /// // create name in namespace
    /// let doc_name = xot.add_name_ns("doc", ns);
    /// let doc_el = xot.new_element(doc_name);
    /// let element = xot.element_mut(doc_el).unwrap();
    ///
    /// // set up namepace prefix in element so it serializes to XML nicely
    /// element.set_prefix(ex, ns);
    ///
    /// let root = xot.new_root(doc_el)?;
    ///
    /// assert_eq!(xot.to_string(root)?, r#"<ex:doc xmlns:ex="http://example.com"/>"#);
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn new_element(&mut self, name: NameId) -> Node {
        let element = Value::Element(Element::new(name));
        self.new_node(element)
    }

    /// Create a new, unattached text node.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<doc/>"#)?;
    /// let doc_el = xot.document_element(root)?;
    /// let txt = xot.new_text("Hello, world!");
    /// xot.append(doc_el, txt)?;
    /// assert_eq!(xot.to_string(root)?, "<doc>Hello, world!</doc>");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn new_text(&mut self, text: &str) -> Node {
        let text = Value::Text(Text::new(text.to_string()));
        self.new_node(text)
    }

    /// Create a new, unattached comment node given comment text.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<doc/>"#)?;
    /// let doc_el = xot.document_element(root)?;
    /// let comment = xot.new_comment("Hello, world!");
    /// xot.append(doc_el, comment)?;
    /// assert_eq!(xot.to_string(root)?, "<doc><!--Hello, world!--></doc>");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn new_comment(&mut self, comment: &str) -> Node {
        let comment = Value::Comment(Comment::new(comment.to_string()));
        self.new_node(comment)
    }

    /// Create a new, unattached processing instruction.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<doc/>"#)?;
    /// let doc_el = xot.document_element(root)?;
    /// let pi = xot.new_processing_instruction("target", Some("data"));
    /// xot.append(doc_el, pi)?;
    /// assert_eq!(xot.to_string(root)?, r#"<doc><?target data?></doc>"#);
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn new_processing_instruction(&mut self, target: &str, data: Option<&str>) -> Node {
        let pi = Value::ProcessingInstruction(ProcessingInstruction::new(
            target.to_string(),
            data.map(|s| s.to_string()),
        ));
        self.new_node(pi)
    }
}
