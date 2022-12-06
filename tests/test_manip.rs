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
    let a = data.name_id("a");

    if let XmlNode::Element(element) = data.xml_node_mut(el_id) {
        element.set_attribute(a, "Changed".to_string());
    }
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc a="Changed"/>"#
    );
}
