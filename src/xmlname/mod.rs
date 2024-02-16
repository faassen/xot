/// This module allows you to use XML names in various ways.
///
/// At first glance this provides an overwhelming amount of ways to handle
/// names, but it turns out there are quite a few use cases. For basic usage
/// you can get away with just studying the [`XmlNameCreate`] and
/// [`XmlNameRef`] types.
///
/// It provides multiple ways to create and access XML names:
//
/// - [`XmlNameCreate`] is a convenient way to create a new name for usage in
///   Xot from strings programmatically. You can use it to create new elements
///   and attributes just like you'd use a `NameId` directly.
///
/// - [`XmlNameRef`] is a reference to a name that is stored by Xot. You
///   can efficiently get a reference to it for a node, or convert a newly
///   created [`XmlNameOwned`] or a stored [`XmlNameState`] into one. It's the
///   most full featured type but it cannot be stored in structs or enums, and
///   cannot be created directly. It implements both the [`NameIdInfo`] and
///   [`NameStrInfo`] traits. You can also use it to create new elements and
///   attributes.
///
/// - [`XmlNameOwned`] is a name that is stored as owned strings. It's handy
///   when you want to handle names in structs or enums without any reference
///   to Xot yet. It's also serde serializable if you enable the `serde`
///   feature. It implements just the [`NameStrInfo`] trait. It's not
///   efficiently storable: for that use [`XmlNameState`].
///
/// - [`XmlNameState`] can be easily and efficiently stored in a struct or enum
///   as it has no reference to Xot, but cannot be created directly and is not
///   easily debuggable or serializable. It implements just the [`NameIdInfo`]
///   trait. You can also use it to create new elements and attributes.
mod create;
mod owned;
mod reference;
mod state;

pub use create::XmlNameCreate;
pub use owned::XmlNameOwned;
pub use reference::{Lookup, NameIdInfo, NameStrInfo, XmlNameRef};
pub use state::XmlNameState;
