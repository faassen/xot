mod document;
mod entity;
mod error;
mod idmap;
mod name;
mod namespace;
mod parse;
mod prefix;
mod serialize;
mod xmlnode;

pub use document::{Document, XmlData};
pub use error::Error;
pub use xmlnode::XmlNode;
