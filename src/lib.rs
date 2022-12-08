#![forbid(unsafe_code)]

//! Xot is an XML library that lets you access manipulate XML documents as a tree in memory.
//!  
//! ```rust
//! use xot::XmlData;
//!
//! let mut data = XmlData::new();
//!
//! let doc = data.parse("<p>Example</p>").unwrap();
//! let root = data.document_element(doc).unwrap();
//! let txt = data.first_child(root).unwrap();
//! let txt_value = data.text_mut(txt).unwrap();
//! txt_value.set("Hello, world!");
//!
//! assert_eq!(data.serialize_to_string(doc), "<p>Hello, world!</p>");
//! ```

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
pub use xmlvalue::{Comment, Element, ProcessingInstruction, Text, Value, ValueType};
pub use xotdata::{Node, XmlData};
