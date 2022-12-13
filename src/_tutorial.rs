//! # Xot Tutorial
//!
//! Xot is a library for working with XML trees in memory.
//!
//! Let's go through a few use cases:
//!
//! ## Accessing XML data
//!
//! We have some XML and we want to access it:
//!
//! ```rust
//! use xot::Xot;
//!
//! let mut xot = Xot::new();
//!
//! let doc = xot.parse("<p>Example</p>").unwrap();
//! let doc_el = xot.document_element(doc).unwrap();
//! let txt_node = xot.first_child(doc_el).unwrap();
//! let txt_value = xot.text(txt_node).unwrap();
//! assert_eq!(txt_value.get(), "Example");
//! ```
//!
//!
