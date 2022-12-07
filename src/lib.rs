#![forbid(unsafe_code)]

mod access;
mod creation;
mod document;
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
mod xmldata;
mod xmlvalue;

pub use access::NodeEdge;
pub use error::Error;
pub use xmldata::{Node, XmlData};
pub use xmlvalue::{Comment, Element, ProcessingInstruction, Text, Value};
