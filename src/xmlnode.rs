use ahash::HashMap;
use id_tree::Tree;
use std::borrow::Cow;
use vector_map::VecMap;

use crate::idmap::{IdIndex, IdMap};

pub enum XmlNode<'a> {
    Element(Element<'a>),
    Text(Cow<'a, str>),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct NameId(u16);

impl IdIndex<NameId> for NameId {
    fn to_id(index: usize) -> NameId {
        NameId(index as u16)
    }

    fn from_id(id: NameId) -> usize {
        id.0 as usize
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct NamespaceId(u8);

impl IdIndex<NamespaceId> for NamespaceId {
    fn to_id(index: usize) -> NamespaceId {
        NamespaceId(index as u8)
    }

    fn from_id(id: NamespaceId) -> usize {
        id.0 as usize
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct Name<'a> {
    name: Cow<'a, str>,
    namespace_id: NamespaceId,
}

impl<'a> Name<'a> {
    pub(crate) fn new(name: &'a str, namespace_id: NamespaceId) -> Self {
        Self {
            name: name.into(),
            namespace_id,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct Namespace<'a>(Cow<'a, str>);

impl<'a> Namespace<'a> {
    pub(crate) fn new(namespace_uri: &'a str) -> Self {
        Self(namespace_uri.into())
    }
}

pub(crate) type Attributes<'a> = VecMap<NameId, Cow<'a, str>>;
pub(crate) type Prefixes<'a> = HashMap<Cow<'a, str>, NamespaceId>;

pub struct Element<'a> {
    name_id: NameId,
    attributes: Attributes<'a>,
    prefixes: Prefixes<'a>,
}

impl<'a> Element<'a> {
    pub(crate) fn new(name_id: NameId) -> Self {
        Element {
            name_id,
            attributes: VecMap::new(),
            // should use a prefix vec map by introducing PrefixId
            prefixes: HashMap::default(),
        }
    }

    pub fn get_attributes(&'a self) -> &'a Attributes<'a> {
        &self.attributes
    }

    pub fn get_attributes_mut(&'a mut self) -> &'a mut Attributes<'a> {
        &mut self.attributes
    }

    pub fn get_prefixes(&'a self) -> &'a Prefixes<'a> {
        &self.prefixes
    }

    pub fn get_prefixes_mut(&'a mut self) -> &'a mut Prefixes<'a> {
        &mut self.prefixes
    }

    pub(crate) fn add_prefix(&'a mut self, prefix: Cow<'a, str>, namespace_id: NamespaceId) {
        self.prefixes.insert(prefix, namespace_id);
    }
}

pub(crate) type Namespaces<'a> = IdMap<NamespaceId, Namespace<'a>>;

pub(crate) type Names<'a> = IdMap<NameId, Name<'a>>;

pub(crate) type XmlTree<'a> = Tree<XmlNode<'a>>;

pub struct Document<'a> {
    pub(crate) namespaces: Namespaces<'a>,
    pub(crate) names: Names<'a>,
    pub(crate) tree: XmlTree<'a>,
}

impl<'a> Document<'a> {
    // pub(crate) fn new(namespaces: Namespaces<'a>, names: Names<'a>, tree: XmlTree<'a>) -> Self {
    //     Document {
    //         namespaces,
    //         names, Names::new(),
    //         tree: Tree::new(),
    //     }
    // }
}
