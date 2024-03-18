//! This module allows you to use XML names in various ways.
//!
//! For basic usage you can get study the [`CreateName`] and [`RefName`]
//! types. Use [`OwnedName`] if you want to store a name independently from Xot
//! but easily convert it to a [`RefName`] or [`CreateName`].
//!
//! It provides multiple ways to create and access XML names:
//
//! - [`CreateName`] is a convenient way to create a new name for usage in Xot
//!   from strings programmatically. You can use it to create new elements and
//!   attributes just like you'd use a [`crate::NameId`] directly. If you want
//!   name without a namespace, use [`CreateName::name`]. If you want a
//!   namespaced name, use [`CreateNamespace`] and [`CreateName::namespaced`],
//!   and [`Xot::append_namespace`] to add it to the tree as a prefix.
//!
//! - [`RefName`] is a reference to a name that is stored by Xot. You can
//!   efficiently get a reference to it for a node, or convert a [`OwnedName`]
//!   into one. It's the most full featured type but it cannot be stored in
//!   structs or enums, and cannot be created directly. It implements the
//!   [`NameStrInfo`] trait. You can also use it to create new elements and
//!   attributes just like a [`crate::NameId`].
//!
//! - [`OwnedName`] is a name that is stored as owned strings. It's handy when
//!   you want to handle names in structs or enums without any reference to Xot
//!   yet. It's also serde serializable if you enable the `serde` feature. It
//!   implements the [`NameStrInfo`] trait.
//!
mod create;
mod owned;
mod reference;

#[cfg(doc)]
use crate::Xot;

pub use create::{CreateName, CreateNamespace};
pub use owned::OwnedName;
pub use reference::{NameStrInfo, RefName};
