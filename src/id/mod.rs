// functions to maintain persistent ids for names, prefixes and namespaces
mod idmap;
mod name;
mod namespace;
mod prefix;

pub use name::NameId;
pub(crate) use name::{Name, NameLookup};
pub use namespace::NamespaceId;
pub(crate) use namespace::NamespaceLookup;
pub use prefix::PrefixId;
pub(crate) use prefix::PrefixLookup;
