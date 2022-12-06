use indextree::{Arena, NodeEdge, NodeId};

use crate::document::Document;
use crate::error::Error;
use crate::name::{Name, NameId, NameLookup};
use crate::namespace::{Namespace, NamespaceId, NamespaceLookup};
use crate::prefix::{Prefix, PrefixId, PrefixLookup};
use crate::xmlnode::XmlNode;

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

    pub fn append(&mut self, parent: XmlNodeId, child: XmlNodeId) -> Result<(), Error> {
        let xml_node = self.xml_node(parent);
        if matches!(xml_node, XmlNode::Root | XmlNode::Element(_)) {
            // XXX also check whether prefixes are valid
            parent.0.checked_append(child.0, self.arena_mut())?;
            Ok(())
        } else {
            Err(Error::InvalidOperation(
                "Can only append to elements or document root".into(),
            ))
        }
    }

    pub fn insert_after(
        &mut self,
        node_id: XmlNodeId,
        new_sibling: XmlNodeId,
    ) -> Result<(), Error> {
        let xml_node = self.xml_node(node_id);
        if !matches!(xml_node, XmlNode::Root) {
            node_id
                .0
                .checked_insert_after(new_sibling.0, self.arena_mut())?;
            Ok(())
        } else {
            Err(Error::InvalidOperation(
                "Cannot insert after document root".into(),
            ))
        }
    }

    pub fn insert_before(
        &mut self,
        node_id: XmlNodeId,
        new_sibling: XmlNodeId,
    ) -> Result<(), Error> {
        let xml_node = self.xml_node(node_id);
        if !matches!(xml_node, XmlNode::Root) {
            node_id
                .0
                .checked_insert_before(new_sibling.0, self.arena_mut())?;
            Ok(())
        } else {
            Err(Error::InvalidOperation(
                "Cannot insert before document root".into(),
            ))
        }
    }

    pub fn prepend(&mut self, parent: XmlNodeId, child: XmlNodeId) -> Result<(), Error> {
        let xml_node = self.xml_node(parent);
        if matches!(xml_node, XmlNode::Root | XmlNode::Element(_)) {
            parent.0.checked_prepend(child.0, self.arena_mut())?;
            Ok(())
        } else {
            Err(Error::InvalidOperation(
                "Can only prepend to elements or document root".into(),
            ))
        }
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

    pub fn remove(&mut self, node_id: XmlNodeId) {
        node_id.0.remove_subtree(self.arena_mut());
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
}

impl Default for XmlData {
    fn default() -> Self {
        Self::new()
    }
}
