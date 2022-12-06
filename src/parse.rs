use ahash::HashMap;
use indextree::NodeId;
use xmlparser::{ElementEnd, Token, Tokenizer};

use crate::document::Document;
use crate::entity::parse_predefined_entities;
use crate::error::Error;
use crate::name::{Name, NameId};
use crate::namespace::Namespace;
use crate::prefix::{Prefix, PrefixId};
use crate::xmldata::XmlData;
use crate::xmlnode::{Attributes, Element, NamespaceInfo, Text, ToNamespace, XmlNode};

struct ElementBuilder {
    prefix: String,
    name: String,
    namespace_info: NamespaceInfo,
    attributes: HashMap<(String, String), String>,
}

impl ElementBuilder {
    fn new(prefix: String, name: String) -> Self {
        ElementBuilder {
            prefix,
            name,
            namespace_info: NamespaceInfo::new(),
            attributes: HashMap::default(),
        }
    }

    fn build_attributes(
        &mut self,
        document_builder: &mut DocumentBuilder,
    ) -> Result<Attributes, Error> {
        let mut attributes = Attributes::new();
        for ((prefix, name), value) in self.attributes.drain() {
            let name_id = document_builder.name_id_builder.attribute_name_id(
                prefix,
                name,
                document_builder.data,
            )?;
            attributes.insert(name_id, value);
        }
        Ok(attributes)
    }

    fn into_element(mut self, document_builder: &mut DocumentBuilder) -> Result<Element, Error> {
        document_builder
            .name_id_builder
            .push(&self.namespace_info.to_namespace);
        let attributes = self.build_attributes(document_builder)?;
        let name_id = document_builder.name_id_builder.element_name_id(
            self.prefix,
            self.name,
            document_builder.data,
        )?;
        Ok(Element {
            name_id,
            namespace_info: self.namespace_info,
            attributes,
        })
    }
}

struct DocumentBuilder<'a> {
    data: &'a mut XmlData,
    tree: NodeId,
    current_node_id: NodeId,
    name_id_builder: NameIdBuilder,
    element_builder: Option<ElementBuilder>,
}

impl<'a> DocumentBuilder<'a> {
    fn new(data: &'a mut XmlData) -> Self {
        let root = data.arena.new_node(XmlNode::Root);
        let mut name_id_builder = NameIdBuilder::new();
        let mut base_to_namespace = ToNamespace::new();
        base_to_namespace.insert(data.empty_prefix_id, data.no_namespace_id);
        name_id_builder.push(&base_to_namespace);
        DocumentBuilder {
            data,
            tree: root,
            current_node_id: root,
            name_id_builder,
            element_builder: None,
        }
    }

    fn into_document(self) -> Document {
        Document { tree: self.tree }
    }

    fn element(&mut self, prefix: &str, name: &str) {
        self.element_builder = Some(ElementBuilder::new(prefix.to_string(), name.to_string()));
    }

    fn prefix(&mut self, prefix: &'a str, namespace_uri: &'a str) {
        let prefix_id = self.data.prefix_lookup.get_id(Prefix::new(prefix.into()));
        let namespace_id = self
            .data
            .namespace_lookup
            .get_id(Namespace::new(namespace_uri.into()));
        self.element_builder
            .as_mut()
            .unwrap()
            .namespace_info
            .add(prefix_id, namespace_id);
    }

    fn attribute(&mut self, prefix: &'a str, name: &'a str, value: &'a str) -> Result<(), Error> {
        self.element_builder
            .as_mut()
            .unwrap()
            .attributes
            .insert((prefix.into(), name.into()), value.into());
        Ok(())
    }

    fn add(&mut self, xml_node: XmlNode) -> NodeId {
        let node_id = self.data.arena.new_node(xml_node);
        self.current_node_id.append(node_id, &mut self.data.arena);
        node_id
    }

    fn open_element(&mut self) -> Result<(), Error> {
        let element_builder = self.element_builder.take().unwrap();
        let element = XmlNode::Element(element_builder.into_element(self)?);
        let node_id = self.add(element);
        self.current_node_id = node_id;
        Ok(())
    }

    fn text(&mut self, content: &str) -> Result<(), Error> {
        let content = parse_predefined_entities(content.into())?;
        self.add(XmlNode::Text(Text::new(content.to_string())));
        Ok(())
    }

    fn close_element_immediate(&mut self) {
        let current_node = self.data.arena.get(self.current_node_id).unwrap();
        if let XmlNode::Element(element) = current_node.get() {
            self.name_id_builder
                .pop(&element.namespace_info.to_namespace);
        }
        self.current_node_id = current_node.parent().expect("Cannot close root node");
    }

    fn close_element(&mut self, prefix: &str, name: &str) -> Result<(), Error> {
        let name_id = self.name_id_builder.element_name_id(
            prefix.to_string(),
            name.to_string(),
            self.data,
        )?;
        let current_node = self.data.arena.get(self.current_node_id).unwrap();
        if let XmlNode::Element(element) = current_node.get() {
            if element.name_id != name_id {
                return Err(Error::InvalidCloseTag(prefix.to_string(), name.to_string()));
            }
            self.name_id_builder
                .pop(&element.namespace_info.to_namespace);
        }
        self.current_node_id = current_node.parent().expect("Cannot close root node");
        Ok(())
    }

    fn is_current_node_root(&self) -> bool {
        if let XmlNode::Root = self.data.arena[self.current_node_id].get() {
            true
        } else {
            false
        }
    }
}

struct NameIdBuilder {
    namespace_stack: Vec<ToNamespace>,
}

impl NameIdBuilder {
    fn new() -> Self {
        Self {
            namespace_stack: Vec::new(),
        }
    }

    fn push(&mut self, to_namespace: &ToNamespace) {
        if to_namespace.is_empty() {
            return;
        }
        let entry = if self.namespace_stack.is_empty() {
            to_namespace.clone()
        } else {
            let mut entry = self.top().clone();
            entry.extend(to_namespace);
            entry
        };
        self.namespace_stack.push(entry);
    }

    fn pop(&mut self, to_namespace: &ToNamespace) {
        if to_namespace.is_empty() {
            return;
        }
        self.namespace_stack.pop();
    }

    #[inline]
    fn top(&self) -> &ToNamespace {
        &self.namespace_stack[self.namespace_stack.len() - 1]
    }

    fn element_name_id(
        &mut self,
        prefix: String,
        name: String,
        data: &mut XmlData,
    ) -> Result<NameId, Error> {
        let prefix_clone = prefix.clone();
        let prefix_id = data.prefix_lookup.get_id(Prefix::new(prefix));
        if let Ok(name_id) = self.name_id_with_prefix_id(prefix_id, name, data) {
            Ok(name_id)
        } else {
            Err(Error::UnknownPrefix(prefix_clone))
        }
    }

    fn attribute_name_id(
        &mut self,
        prefix: String,
        name: String,
        data: &mut XmlData,
    ) -> Result<NameId, Error> {
        // an unprefixed attribute is in no namespace, not
        // in the default namespace
        // https://stackoverflow.com/questions/3312390/xml-default-namespaces-for-unqualified-attribute-names
        let prefix_clone = prefix.clone();
        let prefix_id = data.prefix_lookup.get_id(Prefix::new(prefix));
        if prefix_id == data.empty_prefix_id {
            let name = Name::new(name, data.no_namespace_id);
            return Ok(data.name_lookup.get_id(name));
        }
        if let Ok(name_id) = self.name_id_with_prefix_id(prefix_id, name, data) {
            Ok(name_id)
        } else {
            Err(Error::UnknownPrefix(prefix_clone))
        }
    }

    fn name_id_with_prefix_id(
        &mut self,
        prefix_id: PrefixId,
        name: String,
        data: &mut XmlData,
    ) -> Result<NameId, ()> {
        let namespace_id = if !self.namespace_stack.is_empty() {
            self.top().get(&prefix_id)
        } else {
            None
        };
        let namespace_id = namespace_id.ok_or(())?;
        let name = Name::new(name, *namespace_id);
        Ok(data.name_lookup.get_id(name))
    }
}

impl Document {
    pub fn parse(xml: &str, data: &mut XmlData) -> Result<Self, Error> {
        use Token::*;

        let mut builder = DocumentBuilder::new(data);

        for token in Tokenizer::from(xml) {
            match token? {
                Attribute {
                    prefix,
                    local,
                    value,
                    span: _,
                } => {
                    if prefix.as_str() == "xmlns" {
                        builder.prefix(local.as_str(), value.as_str());
                    } else if local.as_str() == "xmlns" {
                        builder.prefix("", value.as_str());
                    } else {
                        builder.attribute(prefix.as_str(), local.as_str(), value.as_str())?;
                    }
                }
                Text { text } => {
                    builder.text(text.as_str())?;
                }
                ElementStart {
                    prefix,
                    local,
                    span: _,
                } => {
                    builder.element(prefix.as_str(), local.as_str());
                }
                ElementEnd { end, span: _ } => {
                    use self::ElementEnd::*;

                    match end {
                        Open => {
                            builder.open_element()?;
                        }
                        Close(prefix, local) => {
                            builder.close_element(prefix.as_str(), local.as_str())?;
                        }
                        Empty => {
                            builder.open_element()?;
                            builder.close_element_immediate();
                        }
                    }
                }
                _ => {}
            }
        }

        if builder.is_current_node_root() {
            Ok(builder.into_document())
        } else {
            Err(Error::UnclosedTag)
        }
    }
}
