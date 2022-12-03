use ahash::HashMap;
use indextree::NodeId;
use std::borrow::Cow;
use xmlparser::{ElementEnd, Token, Tokenizer};

use crate::document::{namespace_by_prefix, Document, XmlData};
use crate::error::Error;
use crate::name::{Name, NameId};
use crate::namespace::{Namespace, NamespaceId};
use crate::prefix::{Prefix, PrefixId};
use crate::xmlnode::{Attributes, Element, NamespaceInfo, ToNamespace, XmlNode};

struct ElementBuilder<'a> {
    prefix: Cow<'a, str>,
    name: Cow<'a, str>,
    namespace_info: NamespaceInfo,
    attributes: HashMap<(Cow<'a, str>, Cow<'a, str>), Cow<'a, str>>,
}

impl<'a> ElementBuilder<'a> {
    fn new(prefix: Cow<'a, str>, name: Cow<'a, str>) -> Self {
        ElementBuilder {
            prefix,
            name,
            namespace_info: NamespaceInfo::new(),
            attributes: HashMap::default(),
        }
    }

    fn build_attributes(
        &mut self,
        document_builder: &mut DocumentBuilder<'a>,
    ) -> Result<Attributes<'a>, Error> {
        let mut attributes = Attributes::new();
        for ((prefix, name), value) in self.attributes.drain() {
            let name_id =
                document_builder
                    .name_id_builder
                    .name_id(prefix, name, document_builder.data)?;
            attributes.insert(name_id, value);
        }
        Ok(attributes)
    }

    fn into_element(
        mut self,
        document_builder: &mut DocumentBuilder<'a>,
    ) -> Result<Element<'a>, Error> {
        document_builder
            .name_id_builder
            .push(&self.namespace_info.to_namespace);
        let attributes = self.build_attributes(document_builder)?;
        let name_id = document_builder.name_id_builder.name_id(
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
    data: &'a mut XmlData<'a>,
    tree: NodeId,
    current_node_id: NodeId,
    name_id_builder: NameIdBuilder,
    element_builder: Option<ElementBuilder<'a>>,
}

impl<'a> DocumentBuilder<'a> {
    fn new(data: &'a mut XmlData<'a>) -> Self {
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

    fn into_document(self) -> Document<'a> {
        Document {
            data: self.data,
            tree: self.tree,
        }
    }

    fn element(&mut self, prefix: Cow<'a, str>, name: Cow<'a, str>) {
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
        let current_node = self.data.arena.get(self.current_node_id).unwrap();
        if let XmlNode::Element(element) = current_node.get() {
            self.name_id_builder
                .pop(&element.namespace_info.to_namespace);
        }
        self.current_node_id = current_node.parent().expect("Cannot close root node");
    }

    fn get_name_id(
        &mut self,
        prefix: Cow<'a, str>,
        name: Cow<'a, str>,
        namespace_info: &'a NamespaceInfo,
    ) -> Result<NameId, Error> {
        let prefix_clone = prefix.clone();
        let prefix_id = self.data.prefix_lookup.get_id(Prefix::new(prefix));
        // XXX this is relatively slow
        // we could instead have a stack of prefix -> namespace
        // much like in the serializer
        let namespace_id = namespace_info
            .to_namespace
            .get(&prefix_id)
            .copied()
            .or_else(|| self.namespace_by_prefix(prefix_id));
        let namespace_id =
            namespace_id.ok_or_else(|| Error::UnknownPrefix(prefix_clone.into_owned()))?;
        let name = Name::new(name, namespace_id);
        Ok(self.data.name_lookup.get_id(name))
    }

    fn get_attribute_name_id(
        &mut self,
        prefix: Cow<'a, str>,
        name: Cow<'a, str>,
        namespace_info: &'a NamespaceInfo,
    ) -> Result<NameId, Error> {
        let prefix_clone = prefix.clone();
        let prefix_id = self.data.prefix_lookup.get_id(Prefix::new(prefix));
        // an unprefixed attribute is in no namespace, not
        // in the default namespace
        // https://stackoverflow.com/questions/3312390/xml-default-namespaces-for-unqualified-attribute-names
        if prefix_id == self.data.empty_prefix_id {
            let name = Name::new(name, self.data.no_namespace_id);
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
        let namespace_id =
            namespace_id.ok_or_else(|| Error::UnknownPrefix(prefix_clone.into_owned()))?;
        let name = Name::new(name, namespace_id);
        Ok(self.data.name_lookup.get_id(name))
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

    fn name_id<'a>(
        &mut self,
        prefix: Cow<'a, str>,
        name: Cow<'a, str>,
        data: &mut XmlData<'a>,
    ) -> Result<NameId, Error> {
        let prefix_clone = prefix.clone();
        let prefix_id = data.prefix_lookup.get_id(Prefix::new(prefix));
        let namespace_id = if !self.namespace_stack.is_empty() {
            self.top().get(&prefix_id)
        } else {
            None
        };
        let namespace_id =
            namespace_id.ok_or_else(|| Error::UnknownPrefix(prefix_clone.to_string()))?;
        let name = Name::new(name, *namespace_id);
        Ok(data.name_lookup.get_id(name))
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
                    builder.element(prefix.as_str().into(), local.as_str().into());
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
