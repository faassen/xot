use xot::{Value, XmlData};

#[test]
fn test_manipulate_text() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc>Data</doc>"#).unwrap();
    let text_id = data
        .first_child(data.document_element(doc).unwrap())
        .unwrap();
    if let Value::Text(node) = data.value_mut(text_id) {
        node.set("Changed");
    }
    assert_eq!(data.serialize_to_string(doc), r#"<doc>Changed</doc>"#);
}

#[test]
fn test_manipulate_attribute() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc a="Foo"/>"#).unwrap();
    let el_id = data.document_element(doc).unwrap();
    let a = data.name("a").unwrap();

    if let Value::Element(element) = data.value_mut(el_id) {
        element.set_attribute(a, "Changed".to_string());
    }
    assert_eq!(data.serialize_to_string(doc), r#"<doc a="Changed"/>"#);
}

#[test]
fn test_add_attribute() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc/>"#).unwrap();
    let el_id = data.document_element(doc).unwrap();
    assert!(data.name("a").is_none());
    let a = data.add_name("a");

    if let Value::Element(element) = data.value_mut(el_id) {
        element.set_attribute(a, "Created".to_string());
    }
    assert_eq!(data.serialize_to_string(doc), r#"<doc a="Created"/>"#);
}

#[test]
fn test_manipulate_attribute_ns() {
    let mut data = XmlData::new();
    let doc = data
        .parse(r#"<doc xmlns:ns="http://example.com" ns:a="Foo"/>"#)
        .unwrap();
    let el_id = data.document_element(doc).unwrap();
    let ns = data.namespace("http://example.com").unwrap();
    let a = data.name_ns("a", ns).unwrap();

    if let Value::Element(element) = data.value_mut(el_id) {
        element.set_attribute(a, "Changed".to_string());
    }
    assert_eq!(
        data.serialize_to_string(doc),
        r#"<doc xmlns:ns="http://example.com" ns:a="Changed"/>"#
    );
}

#[test]
fn test_add_attribute_ns() {
    let mut data = XmlData::new();
    let doc = data
        .parse(r#"<doc xmlns:foo="http://example.com"/>"#)
        .unwrap();
    let el_id = data.document_element(doc).unwrap();
    let ns = data.namespace("http://example.com").unwrap();
    assert!(data.name_ns("a", ns).is_none());
    let a = data.add_name_ns("a", ns);

    if let Value::Element(element) = data.value_mut(el_id) {
        element.set_attribute(a, "Created".to_string());
    }
    assert_eq!(
        data.serialize_to_string(doc),
        r#"<doc xmlns:foo="http://example.com" foo:a="Created"/>"#
    );
}

#[test]
fn test_append_element() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc/>"#).unwrap();
    let el_id = data.document_element(doc).unwrap();
    let name = data.add_name("a");
    data.append_element(el_id, name).unwrap();
    assert_eq!(data.serialize_to_string(doc), r#"<doc><a/></doc>"#);
}

#[test]
fn test_prepend_element() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc><b/></doc>"#).unwrap();
    let el_id = data.document_element(doc).unwrap();
    let name = data.add_name("a");
    let new_el_id = data.new_element(name);
    data.prepend(el_id, new_el_id).unwrap();
    assert_eq!(data.serialize_to_string(doc), r#"<doc><a/><b/></doc>"#);
}

#[test]
fn test_insert_before_element() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc><b/></doc>"#).unwrap();
    let el_id = data.document_element(doc).unwrap();
    let before_id = data.first_child(el_id).unwrap();
    let name = data.add_name("a");
    let new_el_id = data.new_element(name);
    data.insert_before(before_id, new_el_id).unwrap();
    assert_eq!(data.serialize_to_string(doc), r#"<doc><a/><b/></doc>"#);
}

#[test]
fn test_insert_after_element() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc><b/></doc>"#).unwrap();
    let el_id = data.document_element(doc).unwrap();
    let before_id = data.first_child(el_id).unwrap();
    let name = data.add_name("a");
    let new_el_id = data.new_element(name);
    data.insert_after(before_id, new_el_id).unwrap();
    assert_eq!(data.serialize_to_string(doc), r#"<doc><b/><a/></doc>"#);
}

#[test]
fn test_append_text() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc/>"#).unwrap();
    let el_id = data.document_element(doc).unwrap();
    data.append_text(el_id, "Changed").unwrap();
    assert_eq!(data.serialize_to_string(doc), r#"<doc>Changed</doc>"#);
}

#[test]
fn test_cannot_append_under_text() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc>text</doc>"#).unwrap();
    let el_id = data.document_element(doc).unwrap();
    let txt_id = data.first_child(el_id).unwrap();
    assert!(data.append_text(txt_id, "Changed").is_err());
}

#[test]
fn test_append_text_after_text_consolidates_nodes() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc/>"#).unwrap();
    let el_id = data.document_element(doc).unwrap();
    data.append_text(el_id, "Alpha").unwrap();
    data.append_text(el_id, "Beta").unwrap();
    match data.value(data.first_child(el_id).unwrap()) {
        Value::Text(node) => assert_eq!(node.get(), "AlphaBeta"),
        _ => panic!("Expected text node"),
    }
    assert_eq!(data.serialize_to_string(doc), r#"<doc>AlphaBeta</doc>"#);
}

#[test]
fn test_append_text_after_text_consolidates_nodes_direct_append() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc/>"#).unwrap();
    let el_id = data.document_element(doc).unwrap();
    let txt1 = data.new_text("Alpha");
    let txt2 = data.new_text("Beta");
    data.append(el_id, txt1).unwrap();
    data.append(el_id, txt2).unwrap();
    match data.value(data.first_child(el_id).unwrap()) {
        Value::Text(node) => assert_eq!(node.get(), "AlphaBeta"),
        _ => panic!("Expected text node"),
    }
    assert_eq!(data.serialize_to_string(doc), r#"<doc>AlphaBeta</doc>"#);
}

#[test]
fn test_insert_before_consolidate_text() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc>Alpha</doc>"#).unwrap();
    let el_id = data
        .first_child(data.document_element(doc).unwrap())
        .unwrap();
    let txt = data.new_text("Beta");
    data.insert_before(el_id, txt).unwrap();
    assert_eq!(data.text(el_id).map(|n| n.get()), Some("BetaAlpha"));
    assert_eq!(data.serialize_to_string(doc), r#"<doc>BetaAlpha</doc>"#);
}

#[test]
fn test_insert_after_consolidate_text() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc>Alpha</doc>"#).unwrap();
    let el_id = data
        .first_child(data.document_element(doc).unwrap())
        .unwrap();
    let txt = data.new_text("Beta");
    data.insert_after(el_id, txt).unwrap();
    assert_eq!(data.text(el_id).map(|n| n.get()), Some("AlphaBeta"));
    assert_eq!(data.serialize_to_string(doc), r#"<doc>AlphaBeta</doc>"#);
}

#[test]
fn test_prepend_consolidate_text() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc>Alpha</doc>"#).unwrap();
    let el_id = data.document_element(doc).unwrap();
    let txt = data.new_text("Beta");
    data.prepend(el_id, txt).unwrap();
    let text_el_id = data.first_child(el_id).unwrap();
    assert_eq!(data.text(text_el_id).map(|n| n.get()), Some("BetaAlpha"));
    assert_eq!(data.serialize_to_string(doc), r#"<doc>BetaAlpha</doc>"#);
}

#[test]
fn test_root_node_can_have_only_single_element_append() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc/>"#).unwrap();
    let name = data.add_name("a");
    assert!(data.append_element(doc, name).is_err());
}

#[test]
fn test_root_node_can_have_only_single_element_insert_before() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc/>"#).unwrap();
    let el_id = data.document_element(doc).unwrap();
    let name = data.add_name("a");
    let new_el_id = data.new_element(name);
    assert!(data.insert_before(el_id, new_el_id).is_err());
}

#[test]
fn test_root_node_append_comment() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc/>"#).unwrap();
    data.append_comment(doc, "hello").unwrap();
    assert_eq!(data.serialize_to_string(doc), r#"<doc/><!--hello-->"#);
}

#[test]
fn test_remove_text_consolidation() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc>Alpha<a/>Beta</doc>"#).unwrap();
    let el_id = data
        .children(data.document_element(doc).unwrap())
        .nth(1)
        .unwrap();
    // we found the a element
    let a = data.name("a").unwrap();
    assert_eq!(data.element(el_id).unwrap().name_id(), a);
    // now we remove it
    data.remove(el_id).unwrap();
    // we should have a single text node
    let text_el_id = data
        .first_child(data.document_element(doc).unwrap())
        .unwrap();
    assert_eq!(data.text_str(text_el_id), Some("AlphaBeta"));
    assert_eq!(data.serialize_to_string(doc), r#"<doc>AlphaBeta</doc>"#);
}

#[test]
fn test_create_missing_prefixes() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<doc></doc>"#).unwrap();
    let root_id = data.document_element(doc).unwrap();
    let ns_id = data.add_namespace("http://example.com");
    let name_id = data.add_name_ns("a", ns_id);
    data.append_element(root_id, name_id).unwrap();
    data.create_missing_prefixes(root_id).unwrap();
    assert_eq!(
        data.serialize_to_string(doc),
        r#"<doc xmlns:n0="http://example.com"><n0:a/></doc>"#
    );
}
