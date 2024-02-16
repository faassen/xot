//! This module allows you to use XML names in various ways.
//!
//! At first glance this provides an overwhelming amount of ways to handle
//! names, but it turns out there are quite a few use cases. For basic usage
//! you can get away with just studying the [`Create`] and
//! [`Ref`] types.
//!
//! It provides multiple ways to create and access XML names:
//
//! - [`Create`] is a convenient way to create a new name for usage in Xot from
//!   strings programmatically. You can use it to create new elements and
//!   attributes just like you'd use a [`crate::NameId`] directly.
//!
//! - [`Ref`] is a reference to a name that is stored by Xot. You can
//!   efficiently get a reference to it for a node, or convert a newly created
//!   [`Owned`] or a stored [`State`] into one. It's the most full
//!   featured type but it cannot be stored in structs or enums, and cannot be
//!   created directly. It implements both the [`NameIdInfo`] and
//!   [`NameStrInfo`] traits. You can also use it to create new elements and
//!   attributes just like a [`crate::NameId`].
//!
//! - [`Owned`] is a name that is stored as owned strings. It's handy when you
//!   want to handle names in structs or enums without any reference to Xot
//!   yet. It's also serde serializable if you enable the `serde` feature. It
//!   implements just the [`NameStrInfo`] trait. It's not as efficiently
//!   storable in application state: for that use [`State`].
//!
//! - [`State`] can be easily and efficiently stored in a struct or enum as it
//!   has no reference to Xot, but cannot be created directly and is not easily
//!   debuggable or serializable. Unlike storing `NameId` directly, it retains
//!   information to the `NamespaceId` as well as the `PrefixId`. It implements
//!   just the [`NameIdInfo`] trait. You can also use it to create new elements
//!   and attributes just like a [`crate::NameId`].
mod create;
mod owned;
mod reference;
mod state;

pub use create::Create;
pub use owned::Owned;
pub use reference::{Lookup, NameIdInfo, NameStrInfo, Ref};
pub use state::State;
