use std::borrow::Cow;
use std::fmt::Debug;
use vector_map::VecMap;

use crate::name::NameId;
use crate::namespace::NamespaceId;
use crate::prefix::PrefixId;

#[derive(Debug)]
pub enum XmlNode<'a> {
    Root,
    Element(Element<'a>),
    Text(Cow<'a, str>),
}

pub(crate) type Attributes<'a> = VecMap<NameId, Cow<'a, str>>;
pub(crate) type ToNamespace = VecMap<PrefixId, NamespaceId>;
pub(crate) type ToPrefix = VecMap<NamespaceId, PrefixId>;

#[derive(Debug)]
pub(crate) struct NamespaceInfo {
    pub(crate) to_namespace: ToNamespace,
    pub(crate) to_prefix: ToPrefix,
}

impl NamespaceInfo {
    pub(crate) fn new() -> Self {
        NamespaceInfo {
            to_namespace: VecMap::new(),
            to_prefix: VecMap::new(),
        }
    }

    pub(crate) fn add(&mut self, prefix_id: PrefixId, namespace_id: NamespaceId) {
        self.to_namespace.insert(prefix_id, namespace_id);
        self.to_prefix.insert(namespace_id, prefix_id);
    }
}

#[derive(Debug)]
pub struct Element<'a> {
    pub(crate) name_id: NameId,
    pub(crate) attributes: Attributes<'a>,
    pub(crate) namespace_info: NamespaceInfo,
}

impl<'a> Element<'a> {
    // pub(crate) fn new(name_id: NameId) -> Self {
    //     Element {
    //         name_id,
    //         attributes: VecMap::new(),
    //         namespace_info: NamespaceInfo::new(),
    //     }
    // }

    pub fn get_attributes(&'a self) -> &'a Attributes<'a> {
        &self.attributes
    }

    pub fn get_attributes_mut(&'a mut self) -> &'a mut Attributes<'a> {
        &mut self.attributes
    }
}
