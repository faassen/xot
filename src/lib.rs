#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Xot is an XML library that lets you access manipulate XML documents as a
//! tree in memory.
//!  
//! ```rust
//! use xot::Xot;
//!
//! let mut xot = Xot::new();
//!
//! let root = xot.parse("<p>Example</p>").unwrap();
//! let doc_el = xot.document_element(root).unwrap();
//! let txt = xot.first_child(doc_el).unwrap();
//! let txt_value = xot.text_mut(txt).unwrap();
//! txt_value.set("Hello, world!");
//!
//! assert_eq!(xot.serialize_to_string(root), "<p>Hello, world!</p>");
//! ```
//!
//! ## Xot approach
//!
//! Xot exposes a single [`Xot`] struct that you use to access, create and
//! manipulate all your XML data. Multiple XML trees can exist in an [`Xot`]
//! struct at the same time, and you're free to move nodes between these trees.
//!
//! [`Node`] is a lightweight handle to a node in the XML tree that you use
//! with Xot for both access and manipulation. To navigate the tree use
//! accessors such as [`Xot::first_child`] or iterators such a
//! [`Xot::children`]. You then use operations such as [`Xot::append`] to
//! manipulate the tree.
//!
//! To access and manipulate XML specific data, you use the [`Value`] for a
//! node. This is an enum that's either an [`Element`], [`Text`], [`Comment`]
//! or [`ProcessingInstruction`], or `Root` (which has no value). You can use
//! [`Xot::value`] to get the [`Value`]. Sometimes it's more handy to use the
//! specific accessors for a value, such a [`Xot::element`] or [`Xot::text`].
//!
//! XML names and namespaces in Xot are referenced by ids. In order to
//! construct or compare an element, you first need to get hold of a name. To
//! access a name, use [`Xot::name`]. To create a new name if necessary, use
//! [`Xot::add_name`].

mod access;
mod creation;
mod entity;
mod error;
mod idmap;
mod manipulation;
mod name;
mod nameaccess;
mod namespace;
mod parse;
mod prefix;
mod serialize;
mod valueaccess;
mod xmlvalue;
mod xotdata;

pub use access::NodeEdge;
pub use error::Error;
pub use name::NameId;
pub use namespace::NamespaceId;
pub use prefix::PrefixId;
pub use xmlvalue::{Comment, Element, ProcessingInstruction, Text, Value, ValueType};
pub use xotdata::{Node, Xot};
