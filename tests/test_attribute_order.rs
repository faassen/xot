use xot::{ParseError, Xot};

#[test]
fn test_attribute_parser_order_is_serialization_order1() {
    let mut xot = Xot::new();
    let text = r#"<doc a="A" b="B"/>"#;
    let doc = xot.parse(text).unwrap();

    assert_eq!(xot.to_string(doc).unwrap(), text);
}

#[test]
fn test_attribute_parser_order_is_serialization_order2() {
    let mut xot = Xot::new();
    let text = r#"<doc b="B" a="A"/>"#;
    let doc = xot.parse(text).unwrap();

    assert_eq!(xot.to_string(doc).unwrap(), text);
}

#[test]
fn test_attribute_insert_order_is_serialization_order1() {
    let mut xot = Xot::new();
    let c = xot.add_name("c");
    let text = r#"<doc a="A" b="B"/>"#;
    let doc = xot.parse(text).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let mut attributes = xot.attributes_mut(doc_el);
    attributes.insert(c, "C".to_string());
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc a="A" b="B" c="C"/>"#);
}

#[test]
fn test_prefix_parser_order_is_serialization_order1() -> Result<(), ParseError> {
    let mut xot = Xot::new();
    let text = r#"<doc xmlns:a="A" xmlns:b="B"/>"#;
    let doc = xot.parse(text).unwrap();

    assert_eq!(xot.to_string(doc).unwrap(), text);
    Ok(())
}

#[test]
fn test_prefix_parser_order_is_serialization_order2() {
    let mut xot = Xot::new();
    let text = r#"<doc xmlns:b="B" xmlns:a="A"/>"#;
    let doc = xot.parse(text).unwrap();

    assert_eq!(xot.to_string(doc).unwrap(), text);
}

#[test]
fn test_atribute_reorder() {
    let mut xot = Xot::new();
    let original_xml = r#"<data name="some" xml:space="preserve"/>"#;
    let root = xot.parse(original_xml).unwrap();
    assert_eq!(original_xml, xot.to_string(root).unwrap());
}
