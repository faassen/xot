#![forbid(unsafe_code)]

mod document;
mod entity;
mod error;
mod idmap;
mod name;
mod namespace;
mod parse;
mod prefix;
mod serialize;
mod xmldata;
mod xmlvalue;

// pub use document::Document;
pub use error::Error;
pub use xmldata::XmlData;
pub use xmlvalue::{Comment, Element, ProcessingInstruction, Text, Value};
