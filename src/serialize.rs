use std::io::Write;

use crate::access::NodeEdge;
use crate::entity::serialize_text;
use crate::error::Error;
use crate::name::NameId;
use crate::namespace::NamespaceId;
use crate::xmldata::{Node, XmlData};
use crate::xmlvalue::{ToPrefix, Value, ValueType};

impl XmlData {
    pub fn serialize(&mut self, node: Node, w: &mut impl Write) {
        let root_element = self.root_element(node);
        self.create_missing_prefixes(root_element).unwrap();
        self.serialize_or_missing_prefix(node, w).unwrap();
    }

    pub fn serialize_fragment(&mut self, node: Node, w: &mut impl Write) {
        let root_element = self.top_element(node);
        self.create_missing_prefixes(root_element).unwrap();
        // let to_prefix = self.prefixes_seen(node);
        self.serialize_node(node, w);
    }

    pub fn serialize_or_missing_prefix(&self, node: Node, w: &mut impl Write) -> Result<(), Error> {
        if self.value_type(node) != ValueType::Root {
            panic!("Can only serialize root nodes");
        }
        self.serialize_node(node, w)
    }

    fn serialize_node(&self, node: Node, w: &mut impl Write) -> Result<(), Error> {
        let mut fullname_serializer = FullnameSerializer::new(self);
        for edge in self.traverse(node) {
            match edge {
                NodeEdge::Start(node) => {
                    self.handle_edge_start(node, w, &mut fullname_serializer)?;
                }
                NodeEdge::End(node) => {
                    self.handle_edge_end(node, w, &mut fullname_serializer)?;
                }
            }
        }
        Ok(())
    }

    pub fn serialize_or_missing_prefix_to_string(&self, node: Node) -> Result<String, Error> {
        let mut buf = Vec::new();
        self.serialize_or_missing_prefix(node, &mut buf)?;
        Ok(String::from_utf8(buf).unwrap())
    }

    pub fn serialize_to_string(&mut self, node: Node) -> String {
        let mut buf = Vec::new();
        self.serialize(node, &mut buf);
        String::from_utf8(buf).unwrap()
    }

    fn handle_edge_start(
        &self,
        node: Node,
        w: &mut impl Write,
        fullname_serializer: &mut FullnameSerializer,
    ) -> Result<(), Error> {
        let value = self.value(node);
        match value {
            Value::Root => {}
            Value::Element(element) => {
                fullname_serializer.push(&element.namespace_info.to_prefix);

                let fullname = fullname_serializer.fullname_or_err(element.name_id)?;

                write!(w, "<{}", fullname)?;
                for (prefix_id, namespace_id) in element.namespace_info.to_namespace.iter() {
                    let namespace = self.namespace_lookup.get_value(*namespace_id);
                    if *prefix_id == self.empty_prefix_id {
                        write!(w, " xmlns=\"{}\"", namespace)?;
                    } else {
                        write!(
                            w,
                            " xmlns:{}=\"{}\"",
                            self.prefix_lookup.get_value(*prefix_id),
                            namespace
                        )?;
                    }
                }
                for (name_id, value) in element.attributes.iter() {
                    let fullname = fullname_serializer.fullname_or_err(*name_id)?;
                    write!(w, " {}=\"{}\"", fullname, serialize_text(value.into()))?;
                }

                if self.first_child(node).is_none() {
                    write!(w, "/>")?;
                } else {
                    write!(w, ">")?;
                }
            }
            Value::Text(text) => {
                write!(w, "{}", serialize_text(text.get().into()))?;
            }
            Value::Comment(comment) => {
                write!(w, "<!--{}-->", comment.get())?;
            }
            Value::ProcessingInstruction(pi) => {
                if let Some(data) = pi.get_data() {
                    write!(w, "<?{} {}?>", pi.get_target(), data)?;
                } else {
                    write!(w, "<?{}?>", pi.get_target())?;
                }
            }
        }
        Ok(())
    }

    fn handle_edge_end(
        &self,
        node: Node,
        w: &mut impl Write,
        fullname_serializer: &mut FullnameSerializer,
    ) -> Result<(), Error> {
        let value = self.value(node);
        if let Value::Element(element) = value {
            if self.first_child(node).is_some() {
                let fullname = fullname_serializer.fullname_or_err(element.name_id)?;
                write!(w, "</{}>", fullname)?;
            }
            fullname_serializer.pop(&element.namespace_info.to_prefix);
        }
        Ok(())
    }
}

pub(crate) struct FullnameSerializer<'a> {
    data: &'a XmlData,
    prefix_stack: Vec<ToPrefix>,
}

pub(crate) enum Fullname {
    Name(String),
    MissingPrefix(NamespaceId),
}

impl<'a> FullnameSerializer<'a> {
    pub(crate) fn new(data: &'a XmlData) -> Self {
        Self::with_to_prefix(&ToPrefix::new(), data)
    }

    pub(crate) fn with_to_prefix(to_prefix: &ToPrefix, data: &'a XmlData) -> Self {
        let prefix_stack = vec![to_prefix.clone()];
        Self { data, prefix_stack }
    }

    pub(crate) fn push(&mut self, to_prefix: &ToPrefix) {
        if to_prefix.is_empty() {
            return;
        }

        let mut entry = self.top().clone();
        entry.extend(to_prefix);
        self.prefix_stack.push(entry);
    }

    pub(crate) fn pop(&mut self, to_prefix: &ToPrefix) {
        if to_prefix.is_empty() {
            return;
        }
        self.prefix_stack.pop();
    }

    #[inline]
    fn top(&self) -> &ToPrefix {
        &self.prefix_stack[self.prefix_stack.len() - 1]
    }

    pub(crate) fn fullname(&self, name_id: NameId) -> Fullname {
        let name = self.data.name_lookup.get_value(name_id);
        if name.namespace_id == self.data.no_namespace_id {
            return Fullname::Name(name.name.to_string());
        }
        let prefix_id = if !self.prefix_stack.is_empty() {
            self.top().get(&name.namespace_id)
        } else {
            None
        };
        if let Some(prefix_id) = prefix_id {
            if *prefix_id == self.data.empty_prefix_id {
                Fullname::Name(name.name.to_string())
            } else {
                let prefix = self.data.prefix_lookup.get_value(*prefix_id);
                Fullname::Name(format!("{}:{}", prefix, name.name))
            }
        } else {
            Fullname::MissingPrefix(name.namespace_id)
        }
    }

    fn fullname_or_err(&self, name_id: NameId) -> Result<String, Error> {
        match self.fullname(name_id) {
            Fullname::Name(name) => Ok(name),
            Fullname::MissingPrefix(namespace_id) => Err(Error::MissingPrefix(namespace_id)),
        }
    }
}
