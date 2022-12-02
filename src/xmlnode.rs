use ahash::HashMap;
use id_tree::Tree;
use std::borrow::Cow;
use vector_map::VecMap;

use crate::name::{NameId, Names};
use crate::namespace::{NamespaceId, Namespaces};

pub enum XmlNode<'a> {
    Element(Element<'a>),
    Text(Cow<'a, str>),
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
