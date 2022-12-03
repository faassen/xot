use ahash::HashMap;
use indextree::NodeId;
use std::borrow::Cow;
use xmlparser::{ElementEnd, Token, Tokenizer};

use crate::document::{namespace_by_prefix, Document, XmlData};
use crate::error::Error;
use crate::name::{Name, NameId};
use crate::namespace::{Namespace, NamespaceId};
use crate::prefix::{Prefix, PrefixId};
use crate::xmlnode::{Attributes, Element, NamespaceInfo, XmlNode};

struct ElementBuilder<'a> {
    prefix: &'a str,
    name: &'a str,
    namespace_info: NamespaceInfo,
    attributes: HashMap<(String, String), Cow<'a, str>>,
}

impl<'a> ElementBuilder<'a> {
    fn new(prefix: &'a str, name: &'a str) -> Self {
        ElementBuilder {
            prefix,
            name,
            namespace_info: NamespaceInfo::new(),
            attributes: HashMap::default(),
        }
    }

    fn get_name_id(&self, document_builder: &mut DocumentBuilder<'a>) -> Result<NameId, Error> {
        document_builder.get_name_id(self.prefix, self.name, &self.namespace_info)
    }

    fn get_attributes(
        &mut self,
        document_builder: &mut DocumentBuilder<'a>,
    ) -> Result<Attributes<'a>, Error> {
        let mut attributes = Attributes::new();
        for ((prefix, name), value) in self.attributes.drain() {
            let name_id =
                document_builder.get_attribute_name_id(prefix, name, &self.namespace_info)?;
            attributes.insert(name_id, value.into());
        }
        Ok(attributes)
    }

    fn into_element(
        mut self,
        document_builder: &mut DocumentBuilder<'a>,
    ) -> Result<Element<'a>, Error> {
        let attributes = self.get_attributes(document_builder)?;
        Ok(Element {
            name_id: self.get_name_id(document_builder)?,
            namespace_info: self.namespace_info,
            attributes,
        })
    }
}

struct DocumentBuilder<'a> {
    data: &'a mut XmlData<'a>,
    tree: NodeId,
    current_node_id: NodeId,
    element_builder: Option<ElementBuilder<'a>>,
}

impl<'a> DocumentBuilder<'a> {
    fn new(data: &'a mut XmlData<'a>) -> Self {
        let root = data.arena.new_node(XmlNode::Root);
        DocumentBuilder {
            data,
            tree: root,
            current_node_id: root,
            element_builder: None,
        }
    }

    fn into_document(self) -> Document<'a> {
        Document {
            data: self.data,
            tree: self.tree,
        }
    }

    fn element(&mut self, prefix: &'a str, name: &'a str) {
        self.element_builder = Some(ElementBuilder::new(prefix, name));
    }

    fn namespace_by_prefix(&self, prefix_id: PrefixId) -> Option<NamespaceId> {
        namespace_by_prefix(self.current_node_id, prefix_id, &self.data.arena).or_else(|| {
            if prefix_id == self.data.empty_prefix_id {
                Some(self.data.no_namespace_id)
            } else {
                None
            }
        })
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

    fn add(&mut self, xml_node: XmlNode<'a>) -> NodeId {
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

    fn text(&mut self, content: &'a str) {
        self.add(XmlNode::Text(content.into()));
    }

    fn close_element(&mut self) {
        let parent_node_id = self
            .data
            .arena
            .get(self.current_node_id)
            .unwrap()
            .parent()
            .expect("Cannot close root node");
        self.current_node_id = parent_node_id;
    }

    fn get_name_id(
        &mut self,
        prefix: &'a str,
        name: &'a str,
        namespace_info: &NamespaceInfo,
    ) -> Result<NameId, Error> {
        let prefix_id = self.data.prefix_lookup.get_id(Prefix::new(prefix.into()));
        // XXX this is relatively slow
        // we could instead have a stack of prefix -> namespace
        // much like in the serializer
        let namespace_id = namespace_info
            .to_namespace
            .get(&prefix_id)
            .copied()
            .or_else(|| self.namespace_by_prefix(prefix_id));
        let namespace_id = namespace_id.ok_or_else(|| Error::UnknownPrefix(prefix.to_string()))?;
        let name = Name::new(name.into(), namespace_id);
        Ok(self.data.name_lookup.get_id(name))
    }

    fn get_attribute_name_id(
        &mut self,
        prefix: String,
        name: String,
        namespace_info: &NamespaceInfo,
    ) -> Result<NameId, Error> {
        // XXX a hack to be able to send it to error later
        let prefix_copy = prefix.clone();
        let prefix_id = self.data.prefix_lookup.get_id(Prefix::new(prefix.into()));
        // an unprefixed attribute is in no namespace, not
        // in the default namespace
        // https://stackoverflow.com/questions/3312390/xml-default-namespaces-for-unqualified-attribute-names
        if prefix_id == self.data.empty_prefix_id {
            let name = Name::new(name.into(), self.data.no_namespace_id);
            return Ok(self.data.name_lookup.get_id(name));
        }
        // XXX this is relatively slow
        // we could instead have a stack of prefix -> namespace
        // much like in the serializer
        let namespace_id = namespace_info
            .to_namespace
            .get(&prefix_id)
            .copied()
            .or_else(|| self.namespace_by_prefix(prefix_id));
        let namespace_id = namespace_id.ok_or(Error::UnknownPrefix(prefix_copy))?;
        let name = Name::new(name.into(), namespace_id);
        Ok(self.data.name_lookup.get_id(name))
    }
}

impl<'a> Document<'a> {
    pub fn parse(xml: &'a str, data: &'a mut XmlData<'a>) -> Result<Self, Error> {
        use Token::*;

        let mut builder = DocumentBuilder::new(data);

        for token in Tokenizer::from(xml) {
            match token? {
                Attribute {
                    prefix,
                    local,
                    value,
                    span,
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
                    builder.text(text.as_str());
                }
                ElementStart {
                    prefix,
                    local,
                    span,
                } => {
                    builder.element(prefix.as_str(), local.as_str());
                }
                ElementEnd { end, span } => {
                    use self::ElementEnd::*;

                    match end {
                        Open => {
                            builder.open_element()?;
                        }
                        Close(prefix, local) => {
                            // XXX check that we're closing the right element
                            builder.close_element();
                        }
                        Empty => {
                            builder.open_element()?;
                            builder.close_element();
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(builder.into_document())
    }
}
