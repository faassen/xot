mod attribute;
mod core;
mod entry;
mod namespace;

pub use attribute::{Attributes, MutableAttributes};
pub(crate) use core::category_predicate;
pub use core::{MutableNodeMap, NodeMap};
pub use entry::Entry;
pub(crate) use namespace::to_prefixes;
pub use namespace::{MutableNamespaces, Namespaces};
