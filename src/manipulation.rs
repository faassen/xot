use crate::xmldata::{Node, XmlData};

use crate::error::Error;
use crate::name::NameId;
use crate::xmlvalue::{Value, ValueType};

/// Manipulation of the tree structure.
///
/// This maintains an XML structure:
/// - There is only one document element under the root node which cannot be removed.
/// - The only other nodes that can exist directly under the root node are comments and processing instructions.
/// - You cannot add a node to a node that is not an element or the
///   root node.
///
/// It also ensures that text nodes are consolidated:
/// two text nodes never appear consecutively. If you
/// add a text node after or before another text node,
/// the text is appended to the existing text node,
/// and the added text node is removed. This also
/// happens if you remove a node causing two text
/// nodes to be adjacent; the second text node is
/// removed.
impl XmlData {
    /// Append a child to the end of the children of the given parent.
    ///
    /// It is now the new last node of the parent.
    pub fn append(&mut self, parent: Node, child: Node) -> Result<(), Error> {
        self.add_structure_check(Some(parent), child)?;
        self.remove_structure_check(child)?;
        self.remove_consolidate_text_nodes(self.previous_sibling(child), self.next_sibling(child));
        if self.add_consolidate_text_nodes(child, self.last_child(parent), None) {
            return Ok(());
        }
        parent.get().checked_append(child.get(), self.arena_mut())?;
        Ok(())
    }

    /// Append a text node to a parent node given text.
    pub fn append_text(&mut self, parent: Node, text: &str) -> Result<(), Error> {
        let text_node_id = self.new_text(text);
        self.append(parent, text_node_id)?;
        Ok(())
    }

    /// Append an element node to a parent node given a name.
    ///
    /// Create a name id using [`XmlData::add_name`] or [`XmlData::add_name_ns`], or
    /// reuse an existing name id using [`XmlData::name`], [`XmlData::name_ns`].
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
        self.remove_structure_check(child)?;
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
    pub fn insert_after(&mut self, reference_node: Node, new_sibling: Node) -> Result<(), Error> {
        self.add_structure_check(self.parent(reference_node), new_sibling)?;
        self.remove_structure_check(new_sibling)?;
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
        self.remove_structure_check(new_sibling)?;
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
    /// This removes the nodes from the XmlData.
    pub fn remove(&mut self, node: Node) -> Result<(), Error> {
        self.remove_structure_check(node)?;
        let prev_node = self.previous_sibling(node);
        let next_node = self.next_sibling(node);
        node.get().remove_subtree(self.arena_mut());
        self.remove_consolidate_text_nodes(prev_node, next_node);
        Ok(())
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
                    return Err(Error::InvalidOperation(
                        "Cannot move extra element under document root".into(),
                    ));
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
                    return Err(Error::InvalidOperation("Cannot remove root element".into()));
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
