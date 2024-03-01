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
//! let root = xot.parse("<p>Example</p>")?;
//! let doc_el = xot.document_element(root)?;
//! let txt = xot.first_child(doc_el).unwrap();
//! let txt_value = xot.text_mut(txt).unwrap();
//! txt_value.set("Hello, world!");
//!
//! assert_eq!(xot.to_string(root)?, "<p>Hello, world!</p>");
//! # Ok::<(), xot::Error>(())
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
//! or [`ProcessingInstruction`], [`Attribute`], [`Namespace`] or `Document`
//! (which has no value). You can use [`Xot::value`] to get the [`Value`].
//! Sometimes it's more handy to use the specific accessors for a value, such a
//! [`Xot::element`] or [`Xot::text`].
//!
//! XML names and namespaces in Xot are referenced by ids. In order to
//! construct or compare an element, you first need to get hold of a name. To
//! access a name, use [`Xot::name`]. To create a new name if necessary, use
//! [`Xot::add_name`]. To construct a name with a namespace, use
//! [`Xot::add_namespace`] and then [`Xot::add_name_ns`]. To create a namespace
//! prefix, use [`Xot::add_prefix`]. You can also use the [`xmlname`] module to
//! manage names; see [`xmlname::CreateName`] for a bunch of convenient ways to
//! create names, for instance.
//!
//! Attributes and namespace access is most conveniently done through the
//! [`Xot::attributes`] and [`Xot::namespaces`] accessors. Manipulation is most
//! conveniently done through their mutable variants [`Xot::attributes_mut`]
//! and [`Xot::namespaces_mut`].
//!
//! In some cases however you may want to be able to create namespace and
//! attribute nodes directly. This can be done through the
//! [`Xot::new_namespace_node`] and [`Xot::new_attribute_node`] APIs.
//!
//! You can also create Xot nodes from a fixed structure, the [`fixed`]
//! submodule.

mod access;
mod creation;
mod encoding;
mod entity;
mod error;
pub mod fixed;
mod fullname;
mod id;
mod levelorder;
mod manipulation;
mod nameaccess;
mod nodemap;
pub mod output;
mod parse;
mod pretty;
#[cfg(feature = "proptest")]
pub mod proptest;
mod serialize;
mod serializer;
mod unpretty;
mod valueaccess;
pub mod xmlname;
mod xmlvalue;
mod xotdata;

pub use access::{Axis, NodeEdge};
pub use error::Error;
pub use id::{NameId, NamespaceId, PrefixId};
pub use levelorder::LevelOrder;
pub use nodemap::{
    Attributes, Entry, MutableAttributes, MutableNamespaces, MutableNodeMap, Namespaces, NodeMap,
};
pub use parse::{Span, SpanInfo, SpanInfoKey};
pub use serialize::{PrettyOutputToken, SerializeOptions, WithSerializeOptions};
pub use serializer::{Output, OutputToken};
pub use xmlvalue::{
    Attribute, Comment, Element, Namespace, Prefixes, ProcessingInstruction, Text, Value, ValueType,
};
pub use xotdata::{Node, Xot};
