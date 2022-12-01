use ahash::HashMap;
use std::borrow::Cow;
use vector_map::VecMap;

enum XmlNode<'a> {
    Element(Element<'a>),
    Text(Cow<'a, str>),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct NameId(u16);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct NamespaceId(u8);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Name<'a> {
    name: Cow<'a, str>,
    namespace_id: NamespaceId,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Namespace<'a>(Cow<'a, str>);

type Attributes<'a> = VecMap<NameId, Cow<'a, str>>;

struct Element<'a> {
    name_id: NameId,
    attributes: Attributes<'a>,
}

impl<'a> Element<'a> {
    fn new(name_id: NameId) -> Self {
        Element {
            name_id,
            attributes: VecMap::new(),
        }
    }

    pub fn get_attributes(&'a self) -> &'a Attributes<'a> {
        &self.attributes
    }

    pub fn get_attributes_mut(&'a mut self) -> &'a mut Attributes<'a> {
        &mut self.attributes
    }
}

struct Namespaces<'a> {
    namespaces: Vec<Namespace<'a>>,
    namespace_to_id: HashMap<Namespace<'a>, NamespaceId>,
}

impl<'a> Namespaces<'a> {
    fn new() -> Self {
        Namespaces {
            namespaces: Vec::new(),
            namespace_to_id: HashMap::default(),
        }
    }

    fn get_id(&mut self, namespace_uri: &'a str) -> NamespaceId {
        let namespace = Namespace(Cow::Borrowed(namespace_uri));
        let namespace_id = self.namespace_to_id.get(&namespace);
        if let Some(namespace_id) = namespace_id {
            *namespace_id
        } else {
            let namespace_id = NamespaceId(self.namespaces.len() as u8);
            self.namespaces
                .push(Namespace(Cow::Borrowed(namespace_uri)));
            self.namespace_to_id.insert(namespace, namespace_id);
            namespace_id
        }
    }

    #[inline]
    fn get_namespace(&self, namespace_id: NamespaceId) -> &Namespace<'a> {
        &self.namespaces[namespace_id.0 as usize]
    }
}

struct Names<'a> {
    names: Vec<Name<'a>>,
    name_to_id: HashMap<Name<'a>, NameId>,
}

impl<'a> Names<'a> {
    fn new() -> Self {
        Names {
            names: Vec::new(),
            name_to_id: HashMap::default(),
        }
    }

    fn get_id(&mut self, name: &'a str, namespace_id: NamespaceId) -> NameId {
        let name_value = Name {
            name: Cow::Borrowed(name),
            namespace_id,
        };
        let name_id = self.name_to_id.get(&name_value);
        if let Some(name_id) = name_id {
            *name_id
        } else {
            let name_id = NameId(self.names.len() as u16);
            self.names.push(Name {
                name: Cow::Borrowed(name),
                namespace_id,
            });
            self.name_to_id.insert(name_value, name_id);
            name_id
        }
    }

    #[inline]
    fn get_name(&self, name_id: NameId) -> &Name<'a> {
        &self.names[name_id.0 as usize]
    }
}

struct Document<'a> {
    namespaces: Namespaces<'a>,
    names: Names<'a>,
}
