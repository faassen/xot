use xot::{Document, Value, XmlData};

#[test]
fn test_manipulate_text() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc>Data</doc>"#, &mut data).unwrap();
    let text_id = data.first_child(data.root_element(&doc)).unwrap();
    if let Value::Text(node) = data.value_mut(text_id) {
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

    if let Value::Element(element) = data.value_mut(el_id) {
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

    if let Value::Element(element) = data.value_mut(el_id) {
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

    if let Value::Element(element) = data.value_mut(el_id) {
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

    if let Value::Element(element) = data.value_mut(el_id) {
        element.set_attribute(a, "Created".to_string());
    }
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc xmlns:foo="http://example.com" foo:a="Created"/>"#
    );
}

#[test]
fn test_append_element() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc/>"#, &mut data).unwrap();
    let el_id = data.root_element(&doc);
    let name = data.name_mut("a");
    data.append_element(el_id, name).unwrap();
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc><a/></doc>"#
    );
}

#[test]
fn test_prepend_element() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc><b/></doc>"#, &mut data).unwrap();
    let el_id = data.root_element(&doc);
    let name = data.name_mut("a");
    let new_el_id = data.new_element(name);
    data.prepend(el_id, new_el_id).unwrap();
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc><a/><b/></doc>"#
    );
}

#[test]
fn test_insert_before_element() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc><b/></doc>"#, &mut data).unwrap();
    let el_id = data.root_element(&doc);
    let before_id = data.first_child(el_id).unwrap();
    let name = data.name_mut("a");
    let new_el_id = data.new_element(name);
    data.insert_before(before_id, new_el_id).unwrap();
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc><a/><b/></doc>"#
    );
}

#[test]
fn test_insert_after_element() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc><b/></doc>"#, &mut data).unwrap();
    let el_id = data.root_element(&doc);
    let before_id = data.first_child(el_id).unwrap();
    let name = data.name_mut("a");
    let new_el_id = data.new_element(name);
    data.insert_after(before_id, new_el_id).unwrap();
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc><b/><a/></doc>"#
    );
}

#[test]
fn test_append_text() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc/>"#, &mut data).unwrap();
    let el_id = data.root_element(&doc);
    data.append_text(el_id, "Changed").unwrap();
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc>Changed</doc>"#
    );
}

#[test]
fn test_append_text_after_text_consolidates_nodes() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc/>"#, &mut data).unwrap();
    let el_id = data.root_element(&doc);
    data.append_text(el_id, "Alpha").unwrap();
    data.append_text(el_id, "Beta").unwrap();
    match data.value(data.first_child(el_id).unwrap()) {
        Value::Text(node) => assert_eq!(node.get(), "AlphaBeta"),
        _ => panic!("Expected text node"),
    }
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc>AlphaBeta</doc>"#
    );
}

#[test]
fn test_append_text_after_text_consolidates_nodes_direct_append() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc/>"#, &mut data).unwrap();
    let el_id = data.root_element(&doc);
    let txt1 = data.new_text("Alpha");
    let txt2 = data.new_text("Beta");
    data.append(el_id, txt1).unwrap();
    data.append(el_id, txt2).unwrap();
    match data.value(data.first_child(el_id).unwrap()) {
        Value::Text(node) => assert_eq!(node.get(), "AlphaBeta"),
        _ => panic!("Expected text node"),
    }
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc>AlphaBeta</doc>"#
    );
}

#[test]
fn test_insert_before_consolidate_text() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc>Alpha</doc>"#, &mut data).unwrap();
    let el_id = data.first_child(data.root_element(&doc)).unwrap();
    let txt = data.new_text("Beta");
    data.insert_before(el_id, txt).unwrap();
    assert_eq!(data.text(el_id).map(|n| n.get()), Some("BetaAlpha"));
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc>BetaAlpha</doc>"#
    );
}

#[test]
fn test_insert_after_consolidate_text() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc>Alpha</doc>"#, &mut data).unwrap();
    let el_id = data.first_child(data.root_element(&doc)).unwrap();
    let txt = data.new_text("Beta");
    data.insert_after(el_id, txt).unwrap();
    assert_eq!(data.text(el_id).map(|n| n.get()), Some("AlphaBeta"));
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc>AlphaBeta</doc>"#
    );
}

#[test]
fn test_prepend_consolidate_text() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc>Alpha</doc>"#, &mut data).unwrap();
    let el_id = data.root_element(&doc);
    let txt = data.new_text("Beta");
    data.prepend(el_id, txt).unwrap();
    let text_el_id = data.first_child(el_id).unwrap();
    assert_eq!(data.text(text_el_id).map(|n| n.get()), Some("BetaAlpha"));
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc>BetaAlpha</doc>"#
    );
}

#[test]
fn test_root_node_can_have_only_single_element_append() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc/>"#, &mut data).unwrap();
    let root_id = doc.root();
    let name = data.name_mut("a");
    assert!(data.append_element(root_id, name).is_err());
}

#[test]
fn test_root_node_can_have_only_single_element_insert_before() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc/>"#, &mut data).unwrap();
    let el_id = data.root_element(&doc);
    let name = data.name_mut("a");
    let new_el_id = data.new_element(name);
    assert!(data.insert_before(el_id, new_el_id).is_err());
}

#[test]
fn test_root_node_append_comment() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc/>"#, &mut data).unwrap();
    let root_id = doc.root();
    data.append_comment(root_id, "hello").unwrap();
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc/><!--hello-->"#
    );
}

#[test]
fn test_remove_text_consolidation() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc>Alpha<a/>Beta</doc>"#, &mut data).unwrap();
    let el_id = data.children(data.root_element(&doc)).nth(1).unwrap();
    // we found the a element
    let a = data.name("a").unwrap();
    assert_eq!(data.element(el_id).unwrap().name_id(), a);
    // now we remove it
    data.remove(el_id).unwrap();
    // we should have a single text node
    let text_el_id = data.first_child(data.root_element(&doc)).unwrap();
    assert_eq!(data.text_str(text_el_id), Some("AlphaBeta"));
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc>AlphaBeta</doc>"#
    );
}
