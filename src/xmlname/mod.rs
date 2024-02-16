/// This module allows you to use XML names in various ways.
///
/// It provides three ways to create and access XML names:
///
///  - [`XmlNameOwned`] is a name that is stored as owned strings. It's handy
///   when you want to create a new name or for debugging, or store a name in
///   structs or enums without any reference to Xot yet. It's also serde
///   serializable if you enable the `serde` feature. It implements just the
///   [`NameStrInfo`] trait. It's not efficiently storable: for that use
///   [`XmlNameState`].
/// - [`XmlNameRef`] is a reference to a name that is stored in the Xot. You
///   can efficiently get a reference to it for a node, or convert a newly
///   created [`XmlNameOwned`] or a stored [`XmlNameState`] into one. It's the
///   most full featured type but it cannot be stored in structs or enums. It
///   implements both the [`NameIdInfo`] and [`NameStrInfo`] traits.
/// - [`XmlNameState`] can be easily and efficiently stored in a struct or enum
///   as it has no reference to Xot, but cannot be created directly and is not
///   easily debuggable or serializable. It implements just the [`NameIdInfo`]
///   trait.
///
/// You can convert between these types using `to_ref`, `to_state` and
/// `to_owned` methods.
mod owned;
mod reference;
mod state;

pub use owned::XmlNameOwned;
pub use reference::{NameIdInfo, NameStrInfo, XmlNameRef};
pub use state::XmlNameState;
