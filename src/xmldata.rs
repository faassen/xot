use indextree::{Arena, NodeEdge, NodeId};

use crate::document::Document;
use crate::error::Error;
use crate::name::{Name, NameId, NameLookup};
use crate::namespace::{Namespace, NamespaceId, NamespaceLookup};
use crate::prefix::{Prefix, PrefixId, PrefixLookup};
use crate::xmlnode::{Element, NodeType, Text, XmlNode};

pub type XmlArena = Arena<XmlNode>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct XmlNodeId(NodeId);

impl XmlNodeId {
    pub(crate) fn new(node_id: NodeId) -> Self {
        XmlNodeId(node_id)
    }
    pub(crate) fn get(&self) -> NodeId {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum XmlNodeEdge {
    Start(XmlNodeId),
    End(XmlNodeId),
}

pub struct XmlData {
    pub(crate) arena: XmlArena,
    pub(crate) namespace_lookup: NamespaceLookup,
    pub(crate) prefix_lookup: PrefixLookup,
    pub(crate) name_lookup: NameLookup,
    pub(crate) no_namespace_id: NamespaceId,
    pub(crate) empty_prefix_id: PrefixId,
}

impl XmlData {
    pub fn new() -> Self {
        let mut namespace_lookup = NamespaceLookup::new();
        let no_namespace_id = namespace_lookup.get_id_mut(Namespace::new("".into()));
        let mut prefix_lookup = PrefixLookup::new();
        let empty_prefix_id = prefix_lookup.get_id_mut(Prefix::new("".into()));
        XmlData {
            arena: XmlArena::new(),
            namespace_lookup,
            prefix_lookup,
            name_lookup: NameLookup::new(),
            no_namespace_id,
            empty_prefix_id,
        }
    }

    #[inline]
    pub(crate) fn arena(&self) -> &XmlArena {
        &self.arena
    }

    #[inline]
    pub(crate) fn arena_mut(&mut self) -> &mut XmlArena {
        &mut self.arena
    }

    #[inline]
    pub fn xml_node(&self, node_id: XmlNodeId) -> &XmlNode {
        self.arena[node_id.0].get()
    }

    #[inline]
    pub fn xml_node_mut(&mut self, node_id: XmlNodeId) -> &mut XmlNode {
        self.arena[node_id.0].get_mut()
    }

    // manipulators

    pub(crate) fn new_node(&mut self, xml_node: XmlNode) -> XmlNodeId {
        XmlNodeId(self.arena.new_node(xml_node))
    }

    pub fn new_text(&mut self, text: &str) -> XmlNodeId {
        let text_node = XmlNode::Text(Text::new(text.to_string()));
        self.new_node(text_node)
    }

    pub fn new_element(&mut self, name_id: NameId) -> XmlNodeId {
        let element_node = XmlNode::Element(Element::new(name_id));
        self.new_node(element_node)
    }

    pub fn append(&mut self, parent: XmlNodeId, child: XmlNodeId) -> Result<(), Error> {
        match self.node_type(parent) {
            NodeType::Root => Err(Error::InvalidOperation(
                "Can only append comments or PIs to document root".into(),
            )),
            NodeType::Element => {
                if self.consolidate_text_node(child, self.last_child(parent), None) {
                    return Ok(());
                }
                parent.0.checked_append(child.0, self.arena_mut())?;
                Ok(())
            }
            _ => Err(Error::InvalidOperation(
                "Can only append to elements or document root".into(),
            )),
        }
    }

    pub fn append_text(&mut self, parent: XmlNodeId, text: &str) -> Result<(), Error> {
        let text_node_id = self.new_text(text);
        self.append(parent, text_node_id)?;
        Ok(())
    }

    pub fn append_element(&mut self, parent: XmlNodeId, name_id: NameId) -> Result<(), Error> {
        let element_node_id = self.new_element(name_id);
        self.append(parent, element_node_id)?;
        Ok(())
    }

    pub fn insert_after(
        &mut self,
        reference_node: XmlNodeId,
        new_sibling: XmlNodeId,
    ) -> Result<(), Error> {
        match self.node_type(reference_node) {
            NodeType::Root => Err(Error::InvalidOperation(
                "Cannot insert after document root".into(),
            )),
            _ => {
                if self.consolidate_text_node(
                    new_sibling,
                    Some(reference_node),
                    self.next_sibling(reference_node),
                ) {
                    return Ok(());
                }
                reference_node
                    .0
                    .checked_insert_after(new_sibling.0, self.arena_mut())?;
                Ok(())
            }
        }
    }

    pub fn insert_before(
        &mut self,
        reference_node: XmlNodeId,
        new_sibling: XmlNodeId,
    ) -> Result<(), Error> {
        match self.node_type(reference_node) {
            NodeType::Root => Err(Error::InvalidOperation(
                "Cannot insert before document root".into(),
            )),
            _ => {
                if self.consolidate_text_node(
                    new_sibling,
                    self.previous_sibling(reference_node),
                    Some(reference_node),
                ) {
                    return Ok(());
                }
                reference_node
                    .0
                    .checked_insert_before(new_sibling.0, self.arena_mut())?;
                Ok(())
            }
        }
    }

    pub fn prepend(&mut self, parent: XmlNodeId, child: XmlNodeId) -> Result<(), Error> {
        match self.node_type(parent) {
            NodeType::Root => Err(Error::InvalidOperation(
                "Can only prepend comments or PIs to document root".into(),
            )),
            NodeType::Element => {
                if self.consolidate_text_node(child, None, self.first_child(parent)) {
                    return Ok(());
                }
                parent.0.checked_prepend(child.0, self.arena_mut())?;
                Ok(())
            }
            _ => Err(Error::InvalidOperation(
                "Can only prepend to elements or document root".into(),
            )),
        }
    }

    fn consolidate_text_node(
        &mut self,
        node: XmlNodeId,
        prev_node: Option<XmlNodeId>,
        next_node: Option<XmlNodeId>,
    ) -> bool {
        let added_text = if let XmlNode::Text(t) = self.xml_node(node) {
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
            if let XmlNode::Text(prev) = self.xml_node_mut(prev_node) {
                let mut s = prev.get().to_string();
                s.push_str(&added_text);
                prev.set(s);
                true
            } else {
                false
            }
        } else if let Some(next_node) = next_node {
            if let XmlNode::Text(next) = self.xml_node_mut(next_node) {
                let mut s = added_text;
                s.push_str(next.get());
                next.set(s);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn remove(&mut self, node_id: XmlNodeId) {
        node_id.0.remove_subtree(self.arena_mut());
    }

    // accessors

    pub fn root_element(&self, document: &Document) -> XmlNodeId {
        for child in self.children(document.root()) {
            if let XmlNode::Element(_) = self.xml_node(child) {
                return child;
            }
        }
        unreachable!("Document should always have a single root node")
    }

    pub fn parent(&self, node_id: XmlNodeId) -> Option<XmlNodeId> {
        self.arena()[node_id.0].parent().map(XmlNodeId)
    }

    pub fn first_child(&self, node_id: XmlNodeId) -> Option<XmlNodeId> {
        self.arena()[node_id.0].first_child().map(XmlNodeId)
    }

    pub fn last_child(&self, node_id: XmlNodeId) -> Option<XmlNodeId> {
        self.arena()[node_id.0].last_child().map(XmlNodeId)
    }

    pub fn next_sibling(&self, node_id: XmlNodeId) -> Option<XmlNodeId> {
        self.arena()[node_id.0].next_sibling().map(XmlNodeId)
    }

    pub fn previous_sibling(&self, node_id: XmlNodeId) -> Option<XmlNodeId> {
        self.arena()[node_id.0].previous_sibling().map(XmlNodeId)
    }

    pub fn ancestors(&self, node_id: XmlNodeId) -> impl Iterator<Item = XmlNodeId> + '_ {
        node_id.0.ancestors(self.arena()).map(XmlNodeId)
    }

    pub fn children(&self, node_id: XmlNodeId) -> impl Iterator<Item = XmlNodeId> + '_ {
        node_id.0.children(self.arena()).map(XmlNodeId)
    }

    pub fn reverse_children(&self, node_id: XmlNodeId) -> impl Iterator<Item = XmlNodeId> + '_ {
        node_id.0.reverse_children(self.arena()).map(XmlNodeId)
    }

    pub fn descendants(&self, node_id: XmlNodeId) -> impl Iterator<Item = XmlNodeId> + '_ {
        node_id.0.descendants(self.arena()).map(XmlNodeId)
    }

    pub fn detach(&mut self, node_id: XmlNodeId) {
        node_id.0.detach(self.arena_mut());
    }

    pub fn following_siblings(&self, node_id: XmlNodeId) -> impl Iterator<Item = XmlNodeId> + '_ {
        node_id.0.following_siblings(self.arena()).map(XmlNodeId)
    }

    pub fn preceding_siblings(&self, node_id: XmlNodeId) -> impl Iterator<Item = XmlNodeId> + '_ {
        node_id.0.preceding_siblings(self.arena()).map(XmlNodeId)
    }

    pub fn is_removed(&self, node_id: XmlNodeId) -> bool {
        self.arena()[node_id.0].is_removed()
    }

    pub fn traverse(&self, node_id: XmlNodeId) -> impl Iterator<Item = XmlNodeEdge> + '_ {
        node_id.0.traverse(self.arena()).map(|edge| match edge {
            NodeEdge::Start(node_id) => XmlNodeEdge::Start(XmlNodeId(node_id)),
            NodeEdge::End(node_id) => XmlNodeEdge::End(XmlNodeId(node_id)),
        })
    }

    pub fn reverse_traverse(&self, node_id: XmlNodeId) -> impl Iterator<Item = XmlNodeEdge> + '_ {
        node_id
            .0
            .reverse_traverse(self.arena())
            .map(|edge| match edge {
                NodeEdge::Start(node_id) => XmlNodeEdge::Start(XmlNodeId(node_id)),
                NodeEdge::End(node_id) => XmlNodeEdge::End(XmlNodeId(node_id)),
            })
    }

    pub fn text(&self, node_id: XmlNodeId) -> Option<&str> {
        let xml_node = self.xml_node(node_id);
        if let XmlNode::Text(text) = xml_node {
            Some(text.get())
        } else {
            None
        }
    }

    pub fn element(&self, node_id: XmlNodeId) -> Option<&Element> {
        let xml_node = self.xml_node(node_id);
        if let XmlNode::Element(element) = xml_node {
            Some(element)
        } else {
            None
        }
    }

    pub fn node_type(&self, node_id: XmlNodeId) -> NodeType {
        self.xml_node(node_id).node_type()
    }

    // name & namespace
    pub fn name(&self, name: &str) -> Option<NameId> {
        self.name_ns(name, self.no_namespace_id)
    }

    pub fn name_mut(&mut self, name: &str) -> NameId {
        self.name_ns_mut(name, self.no_namespace_id)
    }

    pub fn name_ns(&self, name: &str, namespace_id: NamespaceId) -> Option<NameId> {
        self.name_lookup
            .get_id(Name::new(name.to_string(), namespace_id))
    }

    pub fn name_ns_mut(&mut self, name: &str, namespace_id: NamespaceId) -> NameId {
        self.name_lookup
            .get_id_mut(Name::new(name.to_string(), namespace_id))
    }

    pub fn namespace(&self, namespace: &str) -> Option<NamespaceId> {
        self.namespace_lookup
            .get_id(Namespace::new(namespace.to_string()))
    }

    pub fn namespace_mut(&mut self, namespace: &str) -> NamespaceId {
        self.namespace_lookup
            .get_id_mut(Namespace::new(namespace.to_string()))
    }
}

impl Default for XmlData {
    fn default() -> Self {
        Self::new()
    }
}
