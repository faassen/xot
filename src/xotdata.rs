use indextree::{Arena, NodeId};

use crate::name::NameLookup;
use crate::namespace::{NamespaceId, NamespaceLookup};
use crate::prefix::{PrefixId, PrefixLookup};
use crate::xmlvalue::Value;

pub(crate) type XmlArena = Arena<Value>;

/// A node in the XML tree.
/// This is a lightweight value and can be copied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Node(NodeId);

impl Node {
    #[inline]
    pub(crate) fn new(node_id: NodeId) -> Self {
        Node(node_id)
    }

    #[inline]
    pub(crate) fn get(&self) -> NodeId {
        self.0
    }
}

/// The `Xot` struct manages all XML tree data in your program. It lets you
/// access and manipulate one or more XML documents and
/// fragments.
///
/// Xot is implemented in several sections focusing on different aspects
/// of accessing and manipulating XML data.
///
/// The Xot structure documentation has different sections:
///
/// * [Read-only access](#read-only-access)
/// * [Creation](#creation)
/// * [Manipulation](#manipulation)
/// * [Names, namespaces and prefixces](#names-namespaces-and-prefixes)
/// * [Parsing](#parsing)
/// * [Serialization](#serialization)
/// * [Value and type access](#value-and-type-access)
pub struct Xot<'a> {
    pub(crate) arena: XmlArena,
    pub(crate) namespace_lookup: NamespaceLookup,
    pub(crate) prefix_lookup: PrefixLookup,
    pub(crate) name_lookup: NameLookup<'a>,
    pub(crate) no_namespace_id: NamespaceId,
    pub(crate) empty_prefix_id: PrefixId,
    pub(crate) xml_namespace_id: NamespaceId,
    pub(crate) xml_prefix_id: PrefixId,
    pub(crate) xml_prefixes: [PrefixId; 1],
    pub(crate) text_consolidation: bool,
}

impl<'a> Xot<'a> {
    /// Create a new `Xot` instance.
    pub fn new() -> Self {
        let mut namespace_lookup = NamespaceLookup::new();
        let no_namespace_id = namespace_lookup.get_id_mut("");
        let mut prefix_lookup = PrefixLookup::new();
        let empty_prefix_id = prefix_lookup.get_id_mut("");
        let xml_namespace_id = namespace_lookup.get_id_mut("http://www.w3.org/XML/1998/namespace");
        let xml_prefix_id = prefix_lookup.get_id_mut("xml");
        Xot {
            arena: XmlArena::new(),
            namespace_lookup,
            prefix_lookup,
            name_lookup: NameLookup::new(),
            no_namespace_id,
            empty_prefix_id,
            xml_namespace_id,
            xml_prefix_id,
            xml_prefixes: [xml_prefix_id],
            text_consolidation: true,
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
}

impl<'a> Default for Xot<'a> {
    fn default() -> Self {
        Self::new()
    }
}
