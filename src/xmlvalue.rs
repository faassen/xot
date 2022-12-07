use std::fmt::Debug;
use vector_map::VecMap;

use crate::name::NameId;
use crate::namespace::NamespaceId;
use crate::prefix::PrefixId;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ValueType {
    Root,
    Element,
    Text,
    ProcessingInstruction,
    Comment,
}

#[derive(Debug)]
pub enum Value {
    Root,
    Element(Element),
    Text(Text),
    Comment(Comment),
    ProcessingInstruction(ProcessingInstruction),
}

impl Value {
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Root => ValueType::Root,
            Value::Element(_) => ValueType::Element,
            Value::Text(_) => ValueType::Text,
            Value::Comment(_) => ValueType::Comment,
            Value::ProcessingInstruction(_) => ValueType::ProcessingInstruction,
        }
    }
}

pub(crate) type Attributes = VecMap<NameId, String>;
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
pub struct Element {
    pub(crate) name_id: NameId,
    pub(crate) attributes: Attributes,
    pub(crate) namespace_info: NamespaceInfo,
}

impl Element {
    pub(crate) fn new(name_id: NameId) -> Self {
        Element {
            name_id,
            attributes: Attributes::new(),
            namespace_info: NamespaceInfo::new(),
        }
    }

    pub fn name_id(&self) -> NameId {
        self.name_id
    }

    pub fn get_attribute(&self, name_id: NameId) -> Option<&str> {
        self.attributes.get(&name_id).map(|s| s.as_str())
    }

    pub fn set_attribute(&mut self, name_id: NameId, value: String) {
        self.attributes.insert(name_id, value);
    }

    // pub fn get_attributes(&'a self) -> &'a Attributes<'a> {
    //     &self.attributes
    // }

    // pub fn get_attributes_mut(&'a mut self) -> &'a mut Attributes<'a> {
    //     &mut self.attributes
    // }
}

#[derive(Debug)]
pub struct Text {
    pub(crate) text: String,
}

impl Text {
    pub(crate) fn new(text: String) -> Self {
        Text { text }
    }

    pub fn get(&self) -> &str {
        &self.text
    }

    pub fn set(&mut self, text: String) {
        self.text = text;
    }
}

#[derive(Debug)]
pub struct Comment {
    pub(crate) text: String,
}

impl Comment {
    pub(crate) fn new(text: String) -> Self {
        Comment { text }
    }

    pub fn get(&self) -> &str {
        &self.text
    }

    pub fn set(&mut self, text: String) {
        self.text = text;
    }
}

#[derive(Debug)]
pub struct ProcessingInstruction {
    pub(crate) target: String,
    pub(crate) data: Option<String>,
}

impl ProcessingInstruction {
    pub(crate) fn new(target: String, data: Option<String>) -> Self {
        ProcessingInstruction { target, data }
    }

    pub fn get_target(&self) -> &str {
        &self.target
    }

    pub fn get_data(&self) -> Option<&str> {
        self.data.as_deref()
    }

    pub fn set_target(&mut self, target: String) {
        self.target = target;
    }

    pub fn set_data(&mut self, data: Option<String>) {
        self.data = data;
    }
}
