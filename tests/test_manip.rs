use xot::{Document, XmlData, XmlNode};

#[test]
fn test_manipulate_text() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc>Data</doc>"#, &mut data).unwrap();
    let text_id = data.first_child(data.root_element(&doc)).unwrap();
    if let XmlNode::Text(node) = data.xml_node_mut(text_id) {
        node.set("Changed".into());
    }
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc>Changed</doc>"#
    );
}

#[test]
fn test_manipulate_attribute() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc a="Foo"/>"#, &mut data).unwrap();
    let el_id = data.root_element(&doc);
    let a = data.name("a").unwrap();

    if let XmlNode::Element(element) = data.xml_node_mut(el_id) {
        element.set_attribute(a, "Changed".to_string());
    }
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc a="Changed"/>"#
    );
}

#[test]
fn test_add_attribute() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc/>"#, &mut data).unwrap();
    let el_id = data.root_element(&doc);
    assert!(data.name("a").is_none());
    let a = data.name_mut("a");

    if let XmlNode::Element(element) = data.xml_node_mut(el_id) {
        element.set_attribute(a, "Created".to_string());
    }
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc a="Created"/>"#
    );
}

#[test]
fn test_manipulate_attribute_ns() {
    let mut data = XmlData::new();
    let doc = Document::parse(
        r#"<doc xmlns:ns="http://example.com" ns:a="Foo"/>"#,
        &mut data,
    )
    .unwrap();
    let el_id = data.root_element(&doc);
    let ns = data.namespace("http://example.com").unwrap();
    let a = data.name_ns("a", ns).unwrap();

    if let XmlNode::Element(element) = data.xml_node_mut(el_id) {
        element.set_attribute(a, "Changed".to_string());
    }
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc xmlns:ns="http://example.com" ns:a="Changed"/>"#
    );
}

#[test]
fn test_add_attribute_ns() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc xmlns:foo="http://example.com"/>"#, &mut data).unwrap();
    let el_id = data.root_element(&doc);
    let ns = data.namespace("http://example.com").unwrap();
    assert!(data.name_ns("a", ns).is_none());
    let a = data.name_ns_mut("a", ns);

    if let XmlNode::Element(element) = data.xml_node_mut(el_id) {
        element.set_attribute(a, "Created".to_string());
    }
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc xmlns:foo="http://example.com" foo:a="Created"/>"#
    );
}
