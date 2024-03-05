use ahash::HashMap;
use std::borrow::Cow;

use crate::error::Error;
use crate::id::{NameId, NamespaceId, PrefixId};
use crate::xmlvalue::Prefixes;
use crate::xotdata::Xot;

type ToPrefixes = HashMap<NamespaceId, Vec<PrefixId>>;

fn inverse_prefixes(prefixes: &Prefixes) -> ToPrefixes {
    let mut to_prefixes = HashMap::default();
    for (prefix, namespace) in prefixes.iter() {
        to_prefixes
            .entry(*namespace)
            .or_insert_with(Vec::new)
            .push(*prefix);
    }
    to_prefixes
}

pub(crate) struct FullnameSerializer<'a> {
    xot: &'a Xot,
    prefix_stack: Vec<(Prefixes, ToPrefixes)>,
}

enum NameInfo<'a> {
    // the name is in the default namespace
    NoNamespace {
        name: &'a str,
    },
    // Prefixes are known for the namespace
    Prefixes {
        name: &'a str,
        namespace_id: NamespaceId,
        prefixes: &'a [PrefixId],
    },
    // the name is in a namespace, but the prefix is not known
    MissingPrefix {
        namespace_id: NamespaceId,
    },
}

pub(crate) enum Fullname<'a> {
    Name(Cow<'a, str>),
    MissingPrefix(NamespaceId),
}

impl<'a> FullnameSerializer<'a> {
    pub(crate) fn new(xot: &'a Xot) -> Self {
        Self::with_prefixes(Prefixes::new(), xot)
    }

    pub(crate) fn with_prefixes(prefixes: Prefixes, xot: &'a Xot) -> Self {
        let to_prefixes = inverse_prefixes(&prefixes);
        let prefix_stack = vec![(prefixes, to_prefixes)];
        Self { xot, prefix_stack }
    }

    pub(crate) fn push(&mut self, prefixes: &Prefixes) {
        let mut entry = self.top_prefixes().clone();
        // add in the new declarations. This may shadow existing prefixes
        entry.extend(prefixes);
        // construct the inverse from this
        let to_prefixes = inverse_prefixes(&entry);
        self.prefix_stack.push((entry, to_prefixes));
    }

    pub(crate) fn pop(&mut self) {
        self.prefix_stack.pop();
    }

    #[inline]
    pub(crate) fn top(&self) -> &(Prefixes, ToPrefixes) {
        &self.prefix_stack[self.prefix_stack.len() - 1]
    }

    #[inline]
    pub(crate) fn top_prefixes(&self) -> &Prefixes {
        &self.top().0
    }

    #[inline]
    pub(crate) fn top_to_prefixes(&self) -> &ToPrefixes {
        &self.top().1
    }

    fn name_info(&self, name_id: NameId) -> NameInfo {
        let name = self.xot.name_lookup.get_value(name_id);
        if name.namespace_id == self.xot.no_namespace_id {
            return NameInfo::NoNamespace { name: &name.name };
        } else if name.namespace_id == self.xot.xml_namespace_id {
            return NameInfo::Prefixes {
                name: name.name.as_ref(),
                namespace_id: name.namespace_id,
                prefixes: &self.xot.xml_prefixes,
            };
        }
        // there should always be at least 1 entry in the stack
        let prefix_ids = self.top_to_prefixes().get(&name.namespace_id);
        if let Some(prefix_ids) = prefix_ids {
            NameInfo::Prefixes {
                name: &name.name,
                namespace_id: name.namespace_id,
                prefixes: prefix_ids,
            }
        } else {
            NameInfo::MissingPrefix {
                namespace_id: name.namespace_id,
            }
        }
    }

    pub(crate) fn element_prefix(&'a self, name_id: NameId) -> Option<PrefixId> {
        match self.name_info(name_id) {
            NameInfo::NoNamespace { .. } => None,
            NameInfo::Prefixes {
                name: _, prefixes, ..
            } => {
                // if any of the prefixes is the empty prefix, prefer that
                if prefixes.iter().any(|p| *p == self.xot.empty_prefix_id) {
                    Some(self.xot.empty_prefix_id)
                } else {
                    // otherwise, use the first prefix
                    Some(prefixes[0])
                }
            }
            NameInfo::MissingPrefix { .. } => None,
        }
    }

    pub(crate) fn fullname(&'a self, name_id: NameId) -> Fullname<'a> {
        match self.name_info(name_id) {
            NameInfo::NoNamespace { name } => Fullname::Name(name.into()),
            NameInfo::Prefixes { name, prefixes, .. } => {
                // if any of the prefixes is the empty prefix, prefer that
                if prefixes.iter().any(|p| *p == self.xot.empty_prefix_id) {
                    Fullname::Name(name.into())
                } else {
                    // otherwise, use the first prefix
                    let prefix = self.xot.prefix_lookup.get_value(prefixes[0]);
                    Fullname::Name(format!("{}:{}", prefix, name).into())
                }
            }
            NameInfo::MissingPrefix { namespace_id } => Fullname::MissingPrefix(namespace_id),
        }
    }

    pub(crate) fn fullname_attr(&'a self, name_id: NameId) -> Fullname<'a> {
        match self.name_info(name_id) {
            NameInfo::NoNamespace { name } => Fullname::Name(name.into()),
            NameInfo::Prefixes {
                name,
                namespace_id,
                prefixes,
            } => {
                // first filter out the empty prefix, as we can't use that for attributes,
                // because attributes without a prefix have no namespace.
                // use the first non-empty prefix, if it exists
                let prefix = prefixes.iter().find(|p| **p != self.xot.empty_prefix_id);
                if let Some(prefix_id) = prefix {
                    let prefix = self.xot.prefix_lookup.get_value(*prefix_id);
                    Fullname::Name(format!("{}:{}", prefix, name).into())
                } else {
                    // otherwise, we can't express the namespace id for the empty prefix
                    Fullname::MissingPrefix(namespace_id)
                }
            }
            NameInfo::MissingPrefix { namespace_id } => Fullname::MissingPrefix(namespace_id),
        }
    }

    pub(crate) fn fullname_or_err(&'a self, name_id: NameId) -> Result<Cow<'a, str>, Error> {
        match self.fullname(name_id) {
            Fullname::Name(name) => Ok(name),
            Fullname::MissingPrefix(namespace_id) => Err(Error::MissingPrefix(
                self.xot.namespace_str(namespace_id).to_string(),
            )),
        }
    }

    pub(crate) fn fullname_attr_or_err(&'a self, name_id: NameId) -> Result<Cow<'a, str>, Error> {
        match self.fullname_attr(name_id) {
            Fullname::Name(name) => Ok(name),
            Fullname::MissingPrefix(namespace_id) => Err(Error::MissingPrefix(
                self.xot.namespace_str(namespace_id).to_string(),
            )),
        }
    }

    pub(crate) fn is_namespace_known(&self, namespace_id: NamespaceId) -> bool {
        self.top_to_prefixes().contains_key(&namespace_id)
    }
}
