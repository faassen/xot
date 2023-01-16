use crate::xotdata::{Node, Xot};

use crate::access::NodeEdge;
use crate::error::Error;
use crate::name::NameId;
use crate::xmlvalue::{Prefixes, Value, ValueType};

/// ## Manipulation
///
/// These methods maintain a well-formed XML structure:
/// - There is only one document element under the root node which cannot be
///   removed.
/// - The only other nodes that can exist directly under the root node are
///   comments and processing instructions.
/// - You cannot add a node to a node that is not an element or the root node.
///
/// Note that you can use these manipulation methods to move nodes between
/// trees -- if you append a node that's in another tree, that node is first
/// detached from the other tree before it's inserted into the new location.
///
/// If text consolidation is enabled (the default), then also ensures that text
/// nodes are consolidated: two text nodes never appear consecutively. If you
/// add a text node after or before another text node, the text is appended to
/// the existing text node, and the added text node is removed. This also
/// happens if you remove a node causing two text nodes to be adjacent; the
/// second text node is removed.
///
/// You can disable and enable text consolidation using [`Xot::set_text_consolidation`].
///
/// Text node consolidation example:
/// ```rust
/// use xot::Xot;
///
/// let mut xot = Xot::new();
/// let root = xot.parse(r#"<doc>First<s/>Second</doc>"#)?;
///
/// let doc_el = xot.document_element(root).unwrap();
/// let children = xot.children(doc_el).collect::<Vec<_>>();
/// let first = children[0];
/// let s = children[1];
/// let second = children[2];
///
/// // Now we remove s from the document
/// xot.remove(s)?;
///
/// // The text nodes are now adjacent, so the second text node is removed
/// // and merged with the first text node.
///
/// let children = xot.children(doc_el).collect::<Vec<_>>();
/// assert_eq!(children.len(), 1);
/// assert_eq!(xot.text_str(children[0]).unwrap(), "FirstSecond");
///
/// # Ok::<(), xot::Error>(())
/// ```
impl<'a> Xot<'a> {
    /// Append a child to the end of the children of the given parent.
    ///
    /// It is now the new last node of the parent.
    ///
    /// Append returns an error if you place a node in a location that is not
    /// allowed, such appending a node to a text node, or appending a new
    /// element to the root (there can be only one document element).
    ///
    /// See also the convenience methods [`Xot::append_element`],
    /// [`Xot::append_text`], [`Xot::append_comment`] and
    /// [`Xot::append_processing_instruction`].
    ///
    /// ```rust
    ///
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    ///
    /// let root = xot.parse(r#"<doc><p>Example</p></doc>"#)?;
    /// let doc_el = xot.document_element(root)?;
    ///
    /// let p_name = xot.add_name("p");
    /// let p_el = xot.new_element(p_name);
    /// xot.append(doc_el, p_el)?;
    ///
    /// assert_eq!(xot.to_string(root)?, r#"<doc><p>Example</p><p/></doc>"#);
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn append(&mut self, parent: Node, child: Node) -> Result<(), Error> {
        self.add_structure_check(Some(parent), child)?;
        self.remove_consolidate_text_nodes(self.previous_sibling(child), self.next_sibling(child));
        if self.add_consolidate_text_nodes(child, self.last_child(parent), None) {
            return Ok(());
        }
        parent.get().checked_append(child.get(), self.arena_mut())?;
        Ok(())
    }

    /// Append a text node to a parent node given text.
    ///
    /// ```rust
    ///
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<doc><p>Example</p></doc>"#)?;
    /// let doc_el = xot.document_element(root)?;
    ///
    /// xot.append_text(doc_el, "Hello")?;
    ///
    /// assert_eq!(xot.to_string(root)?, r#"<doc><p>Example</p>Hello</doc>"#);
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn append_text(&mut self, parent: Node, text: &str) -> Result<(), Error> {
        let text_node_id = self.new_text(text);
        self.append(parent, text_node_id)?;
        Ok(())
    }

    /// Append an element node to a parent node given a name.
    ///
    /// Create a name id using [`Xot::add_name`] or [`Xot::add_name_ns`], or
    /// reuse an existing name id using [`Xot::name`], [`Xot::name_ns`].
    ///
    /// Example:
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    ///
    /// let root = xot.parse(r#"<doc></doc>"#)?;
    /// let doc_el = xot.document_element(root).unwrap();
    ///
    /// let name_id = xot.add_name("foo");
    /// xot.append_element(doc_el, name_id)?;
    ///
    /// assert_eq!(xot.to_string(root)?, "<doc><foo/></doc>");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn append_element(&mut self, parent: Node, name_id: NameId) -> Result<(), Error> {
        let element_node_id = self.new_element(name_id);
        self.append(parent, element_node_id)?;
        Ok(())
    }

    /// Append a comment node to a parent node given comment text.
    pub fn append_comment(&mut self, parent: Node, comment: &str) -> Result<(), Error> {
        let comment_node_id = self.new_comment(comment);
        self.append(parent, comment_node_id)?;
        Ok(())
    }

    /// Append a processing instruction node to a parent node given target and data.
    pub fn append_processing_instruction(
        &mut self,
        parent: Node,
        target: &str,
        data: Option<&str>,
    ) -> Result<(), Error> {
        let pi_node_id = self.new_processing_instruction(target, data);
        self.append(parent, pi_node_id)?;
        Ok(())
    }

    /// Prepend a child to the beginning of the children of the given parent.
    ///
    /// It is now the new first node of the parent.
    pub fn prepend(&mut self, parent: Node, child: Node) -> Result<(), Error> {
        self.add_structure_check(Some(parent), child)?;
        self.remove_consolidate_text_nodes(self.previous_sibling(child), self.next_sibling(child));
        if self.add_consolidate_text_nodes(child, None, self.first_child(parent)) {
            return Ok(());
        }
        parent
            .get()
            .checked_prepend(child.get(), self.arena_mut())?;
        Ok(())
    }

    /// Insert a new sibling after a reference node.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<doc><a/><c/></doc>"#)?;
    ///
    /// let doc_el = xot.document_element(root)?;
    /// let a_el = xot.first_child(doc_el).unwrap();
    ///
    /// let b_name = xot.add_name("b");
    /// let b_el = xot.new_element(b_name);
    ///
    /// xot.insert_after(a_el, b_el)?;
    ///
    /// assert_eq!(xot.to_string(root)?, r#"<doc><a/><b/><c/></doc>"#);
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn insert_after(&mut self, reference_node: Node, new_sibling: Node) -> Result<(), Error> {
        self.add_structure_check(self.parent(reference_node), new_sibling)?;
        self.remove_consolidate_text_nodes(
            self.previous_sibling(new_sibling),
            self.next_sibling(new_sibling),
        );
        if self.add_consolidate_text_nodes(
            new_sibling,
            Some(reference_node),
            self.next_sibling(reference_node),
        ) {
            return Ok(());
        }
        reference_node
            .get()
            .checked_insert_after(new_sibling.get(), self.arena_mut())?;
        Ok(())
    }

    /// Insert a new sibling before a reference node.
    pub fn insert_before(&mut self, reference_node: Node, new_sibling: Node) -> Result<(), Error> {
        self.add_structure_check(self.parent(reference_node), new_sibling)?;
        self.remove_consolidate_text_nodes(
            self.previous_sibling(new_sibling),
            self.next_sibling(new_sibling),
        );
        if self.add_consolidate_text_nodes(
            new_sibling,
            self.previous_sibling(reference_node),
            Some(reference_node),
        ) {
            return Ok(());
        }
        reference_node
            .get()
            .checked_insert_before(new_sibling.get(), self.arena_mut())?;
        Ok(())
    }

    /// Detach a node (and its descendants) from the tree.
    ///
    /// It now becomes a new xml fragment.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<doc><a><b><c/></b></a></doc>"#)?;
    /// let doc_el = xot.document_element(root)?;
    /// let a_el = xot.first_child(doc_el).unwrap();
    ///
    /// xot.detach(a_el)?;
    ///
    /// assert_eq!(xot.to_string(root)?, r#"<doc/>"#);
    /// assert_eq!(xot.to_string(a_el)?, r#"<a><b><c/></b></a>"#);
    ///
    /// // a_al still exist; it's not removed like with [`Xot::remove`].
    /// assert!(!xot.is_removed(a_el));
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn detach(&mut self, node: Node) -> Result<(), Error> {
        self.remove_structure_check(node)?;
        let prev_node = self.previous_sibling(node);
        let next_node = self.next_sibling(node);
        node.get().detach(self.arena_mut());
        self.remove_consolidate_text_nodes(prev_node, next_node);
        Ok(())
    }

    /// Remove a node (and its descendants) from the tree
    ///
    /// This removes the nodes from Xot. Trying to access or
    /// manipulate a removed node results in a panic. You can verify
    /// that a node is removed by using [`Xot::is_removed`].
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<doc><a><b><c/></b></a></doc>"#)?;
    /// let doc_el = xot.document_element(root)?;
    /// let a_el = xot.first_child(doc_el).unwrap();
    ///
    /// xot.remove(a_el)?;
    ///
    /// assert_eq!(xot.to_string(root)?, r#"<doc/>"#);
    ///
    /// // a_al is removed; it's not detached like with [`Xot::detach`].
    /// assert!(xot.is_removed(a_el));
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn remove(&mut self, node: Node) -> Result<(), Error> {
        // we don't do a remove structure check, as we should be able to
        // remove an entire root if we do it explicitly.
        if self.value_type(node) == ValueType::Element && self.is_under_root(node) {
            return Err(Error::InvalidOperation("Cannot remove root element".into()));
        }
        let prev_node = self.previous_sibling(node);
        let next_node = self.next_sibling(node);
        node.get().remove_subtree(self.arena_mut());
        self.remove_consolidate_text_nodes(prev_node, next_node);
        Ok(())
    }

    /// Clone a node and its descendants into a new fragment
    ///
    /// The cloned nodes are not attached to anything. If you clone a root, you
    /// clone the whole document.
    ///
    /// This does not include any namespace prefix information defined in any
    /// ancestors of the cloned node. If you want to preserve such prefix
    /// information, see [`Xot::clone_with_prefixes`].
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<doc><a><b><c/></b></a></doc>"#)?;
    /// let doc_el = xot.document_element(root)?;
    /// let a_el = xot.first_child(doc_el).unwrap();
    ///
    /// let cloned = xot.clone(a_el);
    ///
    /// assert_eq!(xot.to_string(root)?, r#"<doc><a><b><c/></b></a></doc>"#);
    ///
    /// // cloned is not attached to anything
    /// assert!(xot.parent(cloned).is_none());
    ///
    /// // cloned is a new fragment
    /// assert_eq!(xot.to_string(cloned)?, r#"<a><b><c/></b></a>"#);
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn clone(&mut self, node: Node) -> Node {
        let edges = self.traverse(node).collect::<Vec<_>>();

        // we need to create a top node
        let top = if self.is_root(node) {
            // if we clone a root, we need to create a new root
            let value = Value::Root;
            self.new_node(value)
        } else {
            // for anything but the root we create a temporary new element
            let top_name = self.add_name("temporary_root");
            self.new_element(top_name)
        };

        let mut current = top;
        for open_close in edges {
            match open_close {
                NodeEdge::Start(node) => {
                    let value = self.value(node);
                    if value.value_type() == ValueType::Root {
                        continue;
                    }
                    let new_node = self.new_node(value.clone());
                    self.append(current, new_node).unwrap();
                    current = new_node;
                }
                NodeEdge::End(node) => {
                    if self.value_type(node) == ValueType::Root {
                        continue;
                    }
                    current = self.parent(current).unwrap();
                }
            }
        }
        if self.is_root(node) {
            top
        } else {
            // remove the temporary element unless we cloned the root
            let cloned_node = self.first_child(top).unwrap();
            top.get().remove(self.arena_mut());
            cloned_node
        }
    }

    /// Clone a node and its descendants into a new fragment
    ///
    /// If the cloned node is an element, required namespace prefixes that are
    /// in scope are added to the cloned node.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<doc xmlns:foo="http://example.com"><foo:a><foo:b><foo:c/></foo:b></foo:a></doc>"#)?;
    /// let doc_el = xot.document_element(root)?;
    /// let a_el = xot.first_child(doc_el).unwrap();
    ///
    /// let cloned = xot.clone_with_prefixes(a_el);
    /// assert_eq!(xot.to_string(cloned)?, r#"<foo:a xmlns:foo="http://example.com"><foo:b><foo:c/></foo:b></foo:a>"#);
    ///
    /// // if you do a normal clone, prefixes aren't preserved and need to be generated instead
    ///
    /// let cloned = xot.clone(a_el);
    /// xot.create_missing_prefixes(cloned)?;
    /// assert_eq!(xot.to_string(cloned)?, r#"<n0:a xmlns:n0="http://example.com"><n0:b><n0:c/></n0:b></n0:a>"#);
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn clone_with_prefixes(&mut self, node: Node) -> Node {
        // get all prefixes defined in scope
        let prefixes = if let Some(node) = self.parent(node) {
            self.prefixes_in_scope(node)
        } else {
            Prefixes::new()
        };
        let clone = self.clone(node);
        // add any prefixes from outer scope we may need
        if let Some(element) = self.element_mut(clone) {
            for (prefix, ns) in prefixes {
                if element.prefixes().contains_key(&prefix) {
                    continue;
                }
                element.set_prefix(prefix, ns);
            }
        }
        clone
    }

    /// Unwrap an element; its children are moved to its parent.
    ///
    /// The unwrapped element itself is removed.
    ///
    /// You can unwrap the document element, but only if that document
    /// has exactly 1 child that is an element.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    ///
    /// let root = xot.parse(r#"<doc><a><b><c/></b></a></doc>"#)?;
    /// let doc_el = xot.document_element(root)?;
    /// let a_el = xot.first_child(doc_el).unwrap();
    ///
    /// xot.element_unwrap(a_el)?;
    ///
    /// assert_eq!(xot.to_string(root)?, r#"<doc><b><c/></b></doc>"#);
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn element_unwrap(&mut self, node: Node) -> Result<(), Error> {
        if !self.is_element(node) {
            return Err(Error::InvalidOperation(
                "Cannot unwrap non-element nodes".to_string(),
            ));
        }

        if self.is_document_element(node) {
            // unwrapping is possible if the document element contains exactly one child
            // that is an element
            if self.children(node).count() != 1 {
                return Err(Error::InvalidOperation(
                    "Can only unwrap document element if it has exactly 1 element child node"
                        .to_string(),
                ));
            }
            // we now know there is 1 child
            if !self.is_element(self.first_child(node).unwrap()) {
                return Err(Error::InvalidOperation(
                    "Can only unwrap document element if it has exactly 1 element child node"
                        .to_string(),
                ));
            }
        }
        // remove_structure_check is not needed; we already know we don't
        // unwrap the root or non-element child, and document element is
        // taken care of.

        let first_child = self.first_child(node);
        // without children this is like a remove
        if first_child.is_none() {
            return self.remove(node);
        }
        let first_child = first_child.unwrap();
        // there is guaranteed to be a last child if there's a first child
        let last_child = self.last_child(node).unwrap();
        node.get().remove(self.arena_mut());

        let prev_node = self.previous_sibling(first_child);
        let next_node = self.next_sibling(last_child);
        if self.remove_consolidate_text_nodes(prev_node, Some(first_child)) {
            // if first child got consolidated
            if first_child == last_child {
                // if there was only a single child, try to consolidate prev_node with
                // next sibling of last child
                self.remove_consolidate_text_nodes(prev_node, next_node);
            } else {
                // otherwise consolidate last child with next sibling
                self.remove_consolidate_text_nodes(Some(last_child), self.next_sibling(last_child));
            }
        } else {
            // first child did not get consolidated
            self.remove_consolidate_text_nodes(Some(last_child), self.next_sibling(last_child));
        }
        Ok(())
    }

    /// Wrap a node in a new element
    ///
    /// Returns the node for the new wrapping element.
    ///
    /// It's not allowed to wrap the root node. It's allowed to wrap the
    /// document element but not any comment or processing instruction nodes
    /// directly under the root.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<doc><b><c/></b></doc>"#)?;
    /// let doc_el = xot.document_element(root)?;
    /// let b_el = xot.first_child(doc_el).unwrap();
    ///
    /// let a_name = xot.add_name("a");
    /// let wrapper = xot.element_wrap(b_el, a_name)?;
    ///
    /// assert_eq!(xot.to_string(root)?, r#"<doc><a><b><c/></b></a></doc>"#);
    /// assert_eq!(xot.to_string(wrapper)?, r#"<a><b><c/></b></a>"#);
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn element_wrap(&mut self, node: Node, name_id: NameId) -> Result<Node, Error> {
        if self.is_root(node) {
            return Err(Error::InvalidOperation(
                "Cannot wrap document root".to_string(),
            ));
        }
        // we forbid wrapping nodes under the root too unless it's the
        // document element
        if self.is_under_root(node) && !self.is_document_element(node) {
            return Err(Error::InvalidOperation(
                "Cannot wrap nodes under document root except document element".to_string(),
            ));
        }

        if let Some(parent) = self.parent(node) {
            // record previous sibling
            let previous_node = self.previous_sibling(node);
            // create new element
            let wrapper = self.new_element(name_id);
            // detach the node, use low-level detach as we don't want to consolidate
            // text nodes
            node.get().detach(self.arena_mut());
            // append the node to the wrapper
            self.append(wrapper, node)?;
            // now insert the wrapper element
            if let Some(previous_node) = previous_node {
                self.insert_after(previous_node, wrapper)?;
            } else {
                self.prepend(parent, wrapper)?;
            }
            Ok(wrapper)
        } else {
            // we have no parent, standalone node
            let wrapper = self.new_element(name_id);
            self.append(wrapper, node)?;
            Ok(wrapper)
        }
    }

    /// Replace a node with another one.
    ///
    /// The replaced node and all its descendants are removed.
    ///
    /// This works for any node, except the document root itself.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    ///
    /// let root = xot.parse(r#"<doc><a><b/></a><c/></doc>"#)?;
    /// let doc_el = xot.document_element(root)?;
    /// let a_el = xot.first_child(doc_el).unwrap();
    ///
    /// let d_name = xot.add_name("d");
    /// let d_el = xot.new_element(d_name);
    ///
    /// xot.replace(a_el, d_el)?;
    ///
    /// assert_eq!(xot.to_string(root)?, r#"<doc><d/><c/></doc>"#);
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn replace(&mut self, replaced_node: Node, replacing_node: Node) -> Result<(), Error> {
        if self.is_root(replaced_node) {
            return Err(Error::InvalidOperation(
                "Cannot replace document root".to_string(),
            ));
        }
        // there should always be a parent as we're not root
        let parent = self.parent(replaced_node).unwrap();
        // record previous sibling
        let previous_node = self.previous_sibling(replaced_node);
        // remove the replaced node, use low-level remove_tree to avoid
        // text node reconciliation and document element detection
        replaced_node.get().remove_subtree(self.arena_mut());
        // now insert the replacing node
        if let Some(previous_node) = previous_node {
            self.insert_after(previous_node, replacing_node)?;
        } else {
            self.prepend(parent, replacing_node)?;
        }
        Ok(())
    }

    /// Set text consolidation
    ///
    /// By default, text nodes are consolidated when possible. You can turn
    /// off this behavior so text nodes are never merged by calling this.
    pub fn set_text_consolidation(&mut self, consolidate: bool) {
        self.text_consolidation = consolidate;
    }

    fn add_structure_check(&self, parent: Option<Node>, child: Node) -> Result<(), Error> {
        let parent = parent.ok_or_else(|| {
            Error::InvalidOperation("Cannot create siblings for document root".into())
        })?;
        if !matches!(
            self.value_type(parent),
            ValueType::Element | ValueType::Root
        ) {
            return Err(Error::InvalidOperation(
                "Cannot add children to non-element and non-root node".into(),
            ));
        }
        match self.value_type(child) {
            ValueType::Root => {
                return Err(Error::InvalidOperation("Cannot move document root".into()));
            }
            ValueType::Element => {
                if self.is_under_root(child) {
                    return Err(Error::InvalidOperation("Cannot move root element".into()));
                }
                if self.is_root(parent) {
                    for child in self.children(parent) {
                        if self.is_element(child) {
                            return Err(Error::InvalidOperation(
                                "Cannot move extra element under document root".into(),
                            ));
                        }
                    }
                }
            }
            ValueType::Text => {
                if self.is_root(parent) {
                    return Err(Error::InvalidOperation(
                        "Cannot move text under document root".into(),
                    ));
                }
            }
            ValueType::ProcessingInstruction | ValueType::Comment => {
                // these can exist everywhere
            }
        }
        Ok(())
    }

    fn remove_structure_check(&self, node: Node) -> Result<(), Error> {
        match self.value_type(node) {
            ValueType::Root => {
                return Err(Error::InvalidOperation(
                    "Cannot remove document root".into(),
                ));
            }
            ValueType::Element => {
                if self.is_under_root(node) {
                    return Err(Error::InvalidOperation(
                        "Cannot remove document element".into(),
                    ));
                }
            }
            ValueType::Text | ValueType::ProcessingInstruction | ValueType::Comment => {
                // these have no removal constraints
            }
        }
        Ok(())
    }

    fn add_consolidate_text_nodes(
        &mut self,
        node: Node,
        prev_node: Option<Node>,
        next_node: Option<Node>,
    ) -> bool {
        if !self.text_consolidation {
            return false;
        }
        let added_text = if let Value::Text(t) = self.value(node) {
            Some(t.get().to_string())
        } else {
            None
        };
        if added_text.is_none() {
            return false;
        }
        let added_text = added_text.unwrap();

        // due to consolidation, two text nodes can never be adjacent,
        // so consolidate with the previous node or next node is fine
        if let Some(prev_node) = prev_node {
            if let Value::Text(prev) = self.value_mut(prev_node) {
                let mut s = prev.get().to_string();
                s.push_str(&added_text);
                prev.set(s);
                // remove the text node we wanted to insert as it's now consolidated
                node.get().remove(self.arena_mut());
                true
            } else {
                false
            }
        } else if let Some(next_node) = next_node {
            if let Value::Text(next) = self.value_mut(next_node) {
                let mut s = added_text;
                s.push_str(next.get());
                next.set(s);
                // remove the text node we wanted to insert as it's now consolidated
                node.get().remove(self.arena_mut());
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn remove_consolidate_text_nodes(
        &mut self,
        prev_node: Option<Node>,
        next_node: Option<Node>,
    ) -> bool {
        if !self.text_consolidation {
            return false;
        }
        if prev_node.is_none() {
            return false;
        }
        let prev_node = prev_node.unwrap();
        if next_node.is_none() {
            return false;
        }
        let next_node = next_node.unwrap();
        let prev_text = self.text(prev_node);
        let next_text = self.text(next_node);
        if prev_text.is_none() || next_text.is_none() {
            return false;
        }
        let to_add = next_text.unwrap().get().to_string();

        let prev_text_mut = self.text_mut(prev_node).unwrap();
        let mut s = prev_text_mut.get().to_string();
        s.push_str(&to_add);
        prev_text_mut.set(s);
        next_node.get().remove(self.arena_mut());
        true
    }
}
