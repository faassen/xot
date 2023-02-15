//! Proptest support for Xot
//!
//! Proptests allow you to test for *properties* of your code that must hold
//! for arbitrary data. Xot helps you write a proptest by letting you generate
//! an arbitrary XML document.
//!
//! This can be enabled by adding the `proptest` feature to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! xot = { version = "0.9", features = ["proptest"] }
//! ```
//!
//! See the [`proptest`](https://docs.rs/proptest/latest/proptest/)
//! documentation for more information.

use ahash::HashSet;
use proptest::prelude::*;

use crate::fixed::{FixedContent, FixedElement, FixedRoot, FixedRootContent};

const NAMESPACES: &[&str] = &["", "http://example.com/x", "http://example.com/y"];
const PREFIXES: &[&str] = &["", "x", "y"];
const ELEMENT_NAMES: &[&str] = &["a", "b", "c", "d", "e"];
const ATTRIBUTE_NAMES: &[&str] = &["q", "r", "s"];
const PI_NAMES: &[&str] = &["pi1", "pi2", "pi3", "pi4", "pi5"];
const XML_STRING: &str = "[\u{000a}\u{0009}][\u{0020}-\u{D7FF}][\u{E000}-\u{FFFD}]*";
const XML_STRING_WITHOUT_WHITESPACE: &str = "[\u{0020}-\u{D7FF}][\u{E000}-\u{FFFD}]*";

fn arb_attribute() -> impl Strategy<Value = ((String, String), String)> {
    (
        prop::sample::select(ATTRIBUTE_NAMES),
        prop::sample::select(NAMESPACES),
        XML_STRING_WITHOUT_WHITESPACE,
    )
        .prop_map(|(name, namespace, value)| ((name.to_string(), namespace.to_string()), value))
}

fn arb_prefix() -> impl Strategy<Value = (String, String)> {
    (
        prop::sample::select(PREFIXES),
        prop::sample::select(NAMESPACES),
    )
        .prop_map(|(prefix, namespace)| (prefix.to_string(), namespace.to_string()))
}

fn arb_comment() -> impl Strategy<Value = String> {
    XML_STRING.prop_filter("comment", |s| !s.contains('-'))
}

fn arb_processing_instruction() -> impl Strategy<Value = (String, Option<String>)> {
    (
        prop::sample::select(PI_NAMES),
        prop::option::of(
            XML_STRING_WITHOUT_WHITESPACE.prop_filter("non-empty string", |s| !s.is_empty()),
        ),
    )
        .prop_map(|(target, data)| (target.to_string(), data))
}

fn arb_fixed_content() -> impl Strategy<Value = FixedContent> {
    let leaf = prop_oneof![
        XML_STRING.prop_map(FixedContent::Text),
        arb_comment().prop_map(FixedContent::Comment),
        arb_processing_instruction()
            .prop_map(|(target, data)| { FixedContent::ProcessingInstruction(target, data) }),
    ];

    leaf.prop_recursive(
        8,   // levels deep
        256, // maximum size of 256 nodes
        10,  // up to 10 items per collection
        |inner| {
            (
                prop::sample::select(ELEMENT_NAMES),
                prop::sample::select(NAMESPACES),
                prop::collection::vec(inner, 0..10),
                prop::collection::vec(arb_attribute(), 0..4),
                prop::collection::vec(arb_prefix(), 0..4),
            )
                .prop_map(|(name, namespace, children, attributes, prefixes)| {
                    FixedContent::Element(FixedElement {
                        namespace: namespace.to_string(),
                        name: name.to_string(),
                        attributes: unduplicate_attributes(attributes.as_slice()),
                        prefixes: unduplicate_prefixes(prefixes.as_slice()),
                        children,
                    })
                })
        },
    )
}

prop_compose! {
    fn arb_fixed_element()(name in prop::sample::select(ELEMENT_NAMES),
                           namespace in prop::sample::select(NAMESPACES),
                           children in arb_fixed_content(),
                           attributes in prop::collection::vec(arb_attribute(), 0..4),
                           prefixes in prop::collection::vec(arb_prefix(), 0..4)) -> FixedElement {
        FixedElement {
            namespace: namespace.to_string(),
            name: name.to_string(),
            attributes: unduplicate_attributes(attributes.as_slice()),
            prefixes: unduplicate_prefixes(prefixes.as_slice()),
            children: vec![children],
        }
    }
}

fn unduplicate_attributes(
    attributes: &[((String, String), String)],
) -> Vec<((String, String), String)> {
    let mut seen = HashSet::default();
    attributes
        .iter()
        .filter(|((name, namespace), _)| seen.insert((name.clone(), namespace.clone())))
        .cloned()
        .collect()
}

fn unduplicate_prefixes(prefixes: &[(String, String)]) -> Vec<(String, String)> {
    let mut seen = HashSet::default();
    prefixes
        .iter()
        .filter(|(prefix, _)| seen.insert(prefix.clone()))
        .cloned()
        .collect()
}

/// Generate a random XML document.
///
/// This produces a value that can be converted into a `Xot` node using its
/// `xotify` method.
///
/// Example:
///
/// ```notrust
/// use xot::proptest::arb_xml_root;
/// use xot::Xot;
///
/// proptest! {
///   #[test]
///   fn test_arb_xml_can_serialize_parse(root in arb_xml_root()) {
///     let mut xot = Xot::new();
///     let node = root.xotify(&mut xot);
///     let serialized = xot.serialize_to_string(node);
///     let parsed = xot.parse(&serialized);
///     prop_assert!(parsed.is_ok(), "Cannot parse: {} {} {:?}", serialized, parsed.err().unwrap(), serialized);
///   }
/// }
/// ```
pub fn arb_xml_root() -> impl Strategy<Value = FixedRoot> {
    arb_xml_root_with_config(Config {
        comments_and_pi_outside_document_element: true,
    })
}

/// Configure proptest
#[derive(Default)]
pub struct Config {
    /// Can generate comments and pi outside the document element
    comments_and_pi_outside_document_element: bool,
}

/// Generate a random XML document, with configuration.
///
/// This produces a value that can be converted into a `Xot` node using its
/// `xotify` method.
pub fn arb_xml_root_with_config(config: Config) -> BoxedStrategy<FixedRoot> {
    if config.comments_and_pi_outside_document_element {
        let before = prop::collection::vec(
            prop_oneof![
                arb_comment().prop_map(FixedRootContent::Comment),
                arb_processing_instruction().prop_map(|(target, data)| {
                    FixedRootContent::ProcessingInstruction(target, data)
                }),
            ],
            0..10,
        );
        let after = before.clone();
        (before, arb_fixed_element(), after)
            .prop_map(|(before, document_element, after)| FixedRoot {
                before,
                document_element,
                after,
            })
            .boxed()
    } else {
        arb_fixed_element()
            .prop_map(|document_element| FixedRoot {
                before: vec![],
                document_element,
                after: vec![],
            })
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xotdata::Xot;

    proptest! {
        #[test]
        fn test_arb_xml_can_serialize_parse(fixed_root in arb_xml_root()) {
            let mut xot = Xot::new();
            let node = fixed_root.xotify(&mut xot);
            xot.create_missing_prefixes(node).unwrap();
            let serialized = xot.to_string(node).unwrap();
            let parsed = xot.parse(&serialized);
            prop_assert!(parsed.is_ok(), "Cannot parse: {} {} {:?}", serialized, parsed.err().unwrap(), serialized);
        }
    }

    proptest! {
        #[test]
        fn test_arb_xml_can_serialize_parse2(fixed_root in arb_xml_root_with_config(Config::default())) {
            let mut xot = Xot::new();
            let node = fixed_root.xotify(&mut xot);
            xot.create_missing_prefixes(node).unwrap();
            let serialized = xot.to_string(node).unwrap();
            let parsed = xot.parse(&serialized);
            prop_assert!(parsed.is_ok(), "Cannot parse: {} {} {:?}", serialized, parsed.err().unwrap(), serialized);
        }
    }
}
