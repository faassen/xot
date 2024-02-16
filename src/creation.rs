use crate::error::Error;
use crate::id::NameId;
use crate::xmlvalue::{Attribute, Comment, Element, Namespace, ProcessingInstruction, Text, Value};
use crate::xotdata::{Node, Xot};
use crate::{NamespaceId, PrefixId};

/// ## Creation
///
/// These are functions to help create nodes.
///
/// See also the convenience manipulation methods like [`Xot::append_element`]
/// in the manipulation section.
impl Xot {
    pub(crate) fn new_node(&mut self, value: Value) -> Node {
        Node::new(self.arena.new_node(value))
    }

    /// Create a new, unattached document node without document element.
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
    /// /// now create the document
    /// let document = xot.new_document();
    /// xot.append(document, doc_el)?;
    ///
    /// assert_eq!(xot.to_string(document)?, "<doc>Hello, world!</doc>");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn new_document(&mut self) -> Node {
        let root = Value::Document;
        self.new_node(root)
    }

    /// Create a new document node with a document element.
    ///
    /// You can use this to create a new document from scratch. You have to
    /// supply a document element. If you want to create an empty document node,
    /// use `Xot::new_document`.
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
    /// /// now create the document
    /// let document = xot.new_document_with_element(doc_el)?;
    ///
    /// assert_eq!(xot.to_string(document)?, "<doc>Hello, world!</doc>");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn new_document_with_element(&mut self, node: Node) -> Result<Node, Error> {
        if !self.is_element(node) {
            return Err(Error::InvalidOperation(
                "You must supply an element node".to_string(),
            ));
        }
        let document_node = self.new_document();
        self.append(document_node, node)?;
        Ok(document_node)
    }

    /// Create a new, unattached element node given element name.
    ///
    /// You supply a `[crate::NamedId`] or a [`crate::xmlname`] structure that can be turned into
    /// a name id.
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
    /// let root = xot.new_document_with_element(doc_el)?;
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
    ///
    /// // set up namepace prefix for element so it serializes to XML nicely
    /// xot.namespaces_mut(doc_el).insert(ex, ns);
    ///
    /// let root = xot.new_document_with_element(doc_el)?;
    ///
    /// assert_eq!(xot.to_string(root)?, r#"<ex:doc xmlns:ex="http://example.com"/>"#);
    /// # Ok::<(), xot::Error>(())
    /// ```
    ///
    /// Or with `xmlname`:
    ///
    ///```
    /// use xot::{Xot, xmlname, xmlname::NameIdInfo};
    ///
    /// let mut xot = Xot::new();
    ///
    /// let namespace = xmlname::Namespace::new(&mut xot, "ex", "http://example.com");
    /// let doc_name = xmlname::Create::namespaced_name(&mut xot, "doc", &namespace);
    ///
    /// let doc_el = xot.new_element(doc_name);
    ///
    /// // set up namepace prefix for element so it serializes to XML nicely
    /// xot.append_namespace(doc_el, &namespace);
    ///
    /// let root = xot.new_document_with_element(doc_el)?;
    ///
    /// assert_eq!(xot.to_string(root)?, r#"<ex:doc xmlns:ex="http://example.com"/>"#);
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn new_element(&mut self, name: impl Into<NameId>) -> Node {
        let element = Value::Element(Element::new(name.into()));
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
    /// let target = xot.add_name("target");
    /// let root = xot.parse(r#"<doc/>"#)?;
    /// let doc_el = xot.document_element(root)?;
    /// let pi = xot.new_processing_instruction(target, Some("data"));
    /// xot.append(doc_el, pi)?;
    /// assert_eq!(xot.to_string(root)?, r#"<doc><?target data?></doc>"#);
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn new_processing_instruction(
        &mut self,
        target: impl Into<NameId>,
        data: Option<&str>,
    ) -> Node {
        let pi = Value::ProcessingInstruction(ProcessingInstruction::new(
            target.into(),
            data.map(|s| s.to_string()),
        ));
        self.new_node(pi)
    }

    /// Create a new, unattached attribute node.
    ///
    /// You can then use [`Xot::append_attribute_node`] to add it to an element node.
    ///
    /// This method is useful in situations where attributes need to be created
    /// independently of elements, but for many use cases you can use the
    /// [`Xot::attributes_mut`] API to create attributes instead.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let foo = xot.add_name("foo");
    /// let root = xot.parse(r#"<doc/>"#)?;
    /// let doc_el = xot.document_element(root)?;
    /// let attr = xot.new_attribute_node(foo, "FOO".to_string());
    /// xot.append_attribute_node(doc_el, attr)?;
    /// assert_eq!(xot.to_string(root)?, r#"<doc foo="FOO"/>"#);
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn new_attribute_node(&mut self, name: impl Into<NameId>, value: String) -> Node {
        let attr = Value::Attribute(Attribute {
            name_id: name.into(),
            value,
        });
        self.new_node(attr)
    }

    /// Create a new, unattached namespace declaration node.
    ///
    /// You can then use [`Xot::append_namespace_node`] to add it to an element
    /// node.
    ///
    /// This method is useful in situations where namespaces need to be created
    /// independently of elements, but for many use cases you can use the
    /// [`Xot::namespaces_mut`] API to create namespaces instead.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let foo = xot.add_prefix("foo");
    /// let ns = xot.add_namespace("http://example.com");
    /// let root = xot.parse(r#"<doc/>"#)?;
    /// let doc_el = xot.document_element(root)?;
    /// let ns = xot.new_namespace_node(foo, ns);
    /// xot.append_namespace_node(doc_el, ns)?;
    /// assert_eq!(xot.to_string(root)?, r#"<doc xmlns:foo="http://example.com"/>"#);
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn new_namespace_node(&mut self, prefix: PrefixId, namespace: NamespaceId) -> Node {
        let ns = Value::Namespace(Namespace {
            prefix_id: prefix,
            namespace_id: namespace,
        });
        self.new_node(ns)
    }
}
