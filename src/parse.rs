use std::borrow::Cow;

use id_tree::{InsertBehavior, Node, NodeId};
use xmlparser::{ElementEnd, Token, Tokenizer};

use crate::xmlnode::{
    Document, Element, Name, Names, Namespace, NamespaceId, Namespaces, Prefixes, XmlNode, XmlTree,
};

pub enum Error {
    IdTreeError(id_tree::NodeIdError),
    ParserError(xmlparser::Error),
}

impl From<xmlparser::Error> for Error {
    #[inline]
    fn from(e: xmlparser::Error) -> Self {
        Error::ParserError(e)
    }
}

impl From<id_tree::NodeIdError> for Error {
    #[inline]
    fn from(e: id_tree::NodeIdError) -> Self {
        Error::IdTreeError(e)
    }
}

struct DocumentBuilder<'a> {
    namespaces: Namespaces<'a>,
    names: Names<'a>,
    tree: XmlTree<'a>,
    current_node_id: Option<NodeId>,
    current_element: Option<Element<'a>>,
}

impl<'a> DocumentBuilder<'a> {
    fn new() -> Self {
        DocumentBuilder {
            namespaces: Namespaces::new(),
            names: Names::new(),
            tree: XmlTree::new(),
            current_node_id: None,
            current_element: None,
        }
    }

    fn element(&mut self, name: &'a str, prefix: &'a str) {
        // XXX what if prefix is not known
        let namespace_id = self.namespace_by_prefix(prefix);
        if let Some(namespace_id) = namespace_id {
            let name = Name::new(name, namespace_id);
            let name_id = self.names.get_id(name);
            self.current_element = Some(Element::new(name_id));
        }
    }

    fn namespace_by_prefix(&self, prefix: &'a str) -> Option<NamespaceId> {
        // XXX what if there is no current element?
        if let Some(current_element) = &self.current_element {
            current_element.get_prefixes().get(prefix).copied()
            // XXX should chain with parents if not found
        } else {
            None
        }
    }

    fn prefix(&mut self, prefix: &'a str, namespace_uri: &'a str) {
        let namespace_id = self.namespaces.get_id(Namespace::new(namespace_uri));
        // XXX what if there is no current element
        // use prefixes map
        // use current_prefixes VecMap
        // take it when constructing the element
        // so maybe a current element context that we can take() as a whole
        // if let Some(current_element) = &mut self.current_element {
        //     let prefix = prefix.to_string();
        //     current_element.add_prefix(Cow::Owned(prefix), namespace_id);
        // }
    }

    fn add(&mut self, xml_node: XmlNode<'a>) -> Result<(), Error> {
        let behavior = match self.current_node_id {
            Some(ref current_node_id) => InsertBehavior::UnderNode(current_node_id),
            None => InsertBehavior::AsRoot,
        };
        let new_node_id = self.tree.insert(Node::new(xml_node), behavior)?;
        self.current_node_id = Some(new_node_id);
        Ok(())
    }

    fn open_element(&mut self) -> Result<(), Error> {
        let element = self.current_element.take().unwrap();
        self.add(XmlNode::Element(element))?;
        Ok(())
    }

    fn close_element(&mut self) {
        // XXX what if empty without open?
        let current_node_id = self.current_node_id.take().unwrap();
        // we have to clone the node id here, but don't maintain other references
        self.current_node_id = self.tree.get(&current_node_id).unwrap().parent().cloned();
    }
}

impl<'a> Document<'a> {
    fn parse(xml: &'a str) -> Result<Self, Error> {
        use Token::*;

        let mut builder = DocumentBuilder::new();

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
                    } else {
                    }
                }
                Text { text } => {
                    let text_node = XmlNode::Text(text.as_str().into());
                }
                ElementStart {
                    prefix,
                    local,
                    span,
                } => {
                    builder.element(local.as_str(), prefix.as_str());
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
                            builder.close_element();
                        }
                    }
                }
                _ => {}
            }
        }

        todo!();
        // Ok(Document {
        //     namespaces: builder.namespaces,
        //     names: builder.names,
        //     tree: builder.tree,
        // })
    }
}
