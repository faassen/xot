use xot::{Value, Xot};

#[test]
fn test_manipulate_text() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Data</doc>"#).unwrap();
    let text_id = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    if let Value::Text(node) = xot.value_mut(text_id) {
        node.set("Changed");
    }
    assert_eq!(xot.serialize_to_string(doc), r#"<doc>Changed</doc>"#);
}

#[test]
fn test_manipulate_attribute() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc a="Foo"/>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    let a = xot.name("a").unwrap();

    if let Value::Element(element) = xot.value_mut(el_id) {
        element.set_attribute(a, "Changed".to_string());
    }
    assert_eq!(xot.serialize_to_string(doc), r#"<doc a="Changed"/>"#);
}

#[test]
fn test_add_attribute() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc/>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    assert!(xot.name("a").is_none());
    let a = xot.add_name("a");

    if let Value::Element(element) = xot.value_mut(el_id) {
        element.set_attribute(a, "Created".to_string());
    }
    assert_eq!(xot.serialize_to_string(doc), r#"<doc a="Created"/>"#);
}

#[test]
fn test_manipulate_attribute_ns() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc xmlns:ns="http://example.com" ns:a="Foo"/>"#)
        .unwrap();
    let el_id = xot.document_element(doc).unwrap();
    let ns = xot.namespace("http://example.com").unwrap();
    let a = xot.name_ns("a", ns).unwrap();

    if let Value::Element(element) = xot.value_mut(el_id) {
        element.set_attribute(a, "Changed".to_string());
    }
    assert_eq!(
        xot.serialize_to_string(doc),
        r#"<doc xmlns:ns="http://example.com" ns:a="Changed"/>"#
    );
}

#[test]
fn test_add_attribute_ns() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc xmlns:foo="http://example.com"/>"#)
        .unwrap();
    let el_id = xot.document_element(doc).unwrap();
    let ns = xot.namespace("http://example.com").unwrap();
    assert!(xot.name_ns("a", ns).is_none());
    let a = xot.add_name_ns("a", ns);

    if let Value::Element(element) = xot.value_mut(el_id) {
        element.set_attribute(a, "Created".to_string());
    }
    assert_eq!(
        xot.serialize_to_string(doc),
        r#"<doc xmlns:foo="http://example.com" foo:a="Created"/>"#
    );
}

#[test]
fn test_append_element() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc/>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    let name = xot.add_name("a");
    xot.append_element(el_id, name).unwrap();
    assert_eq!(xot.serialize_to_string(doc), r#"<doc><a/></doc>"#);
}

#[test]
fn test_prepend_element() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc><b/></doc>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    let name = xot.add_name("a");
    let new_el_id = xot.new_element(name);
    xot.prepend(el_id, new_el_id).unwrap();
    assert_eq!(xot.serialize_to_string(doc), r#"<doc><a/><b/></doc>"#);
}

#[test]
fn test_insert_before_element() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc><b/></doc>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    let before_id = xot.first_child(el_id).unwrap();
    let name = xot.add_name("a");
    let new_el_id = xot.new_element(name);
    xot.insert_before(before_id, new_el_id).unwrap();
    assert_eq!(xot.serialize_to_string(doc), r#"<doc><a/><b/></doc>"#);
}

#[test]
fn test_insert_after_element() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc><b/></doc>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    let before_id = xot.first_child(el_id).unwrap();
    let name = xot.add_name("a");
    let new_el_id = xot.new_element(name);
    xot.insert_after(before_id, new_el_id).unwrap();
    assert_eq!(xot.serialize_to_string(doc), r#"<doc><b/><a/></doc>"#);
}

#[test]
fn test_append_text() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc/>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    xot.append_text(el_id, "Changed").unwrap();
    assert_eq!(xot.serialize_to_string(doc), r#"<doc>Changed</doc>"#);
}

#[test]
fn test_cannot_append_under_text() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>text</doc>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    let txt_id = xot.first_child(el_id).unwrap();
    assert!(xot.append_text(txt_id, "Changed").is_err());
}

#[test]
fn test_append_text_after_text_consolidates_nodes() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc/>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    xot.append_text(el_id, "Alpha").unwrap();
    xot.append_text(el_id, "Beta").unwrap();
    match xot.value(xot.first_child(el_id).unwrap()) {
        Value::Text(node) => assert_eq!(node.get(), "AlphaBeta"),
        _ => panic!("Expected text node"),
    }
    assert_eq!(xot.serialize_to_string(doc), r#"<doc>AlphaBeta</doc>"#);
}

#[test]
fn test_append_text_after_text_consolidates_nodes_direct_append() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc/>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    let txt1 = xot.new_text("Alpha");
    let txt2 = xot.new_text("Beta");
    xot.append(el_id, txt1).unwrap();
    xot.append(el_id, txt2).unwrap();
    match xot.value(xot.first_child(el_id).unwrap()) {
        Value::Text(node) => assert_eq!(node.get(), "AlphaBeta"),
        _ => panic!("Expected text node"),
    }
    assert_eq!(xot.serialize_to_string(doc), r#"<doc>AlphaBeta</doc>"#);
}

#[test]
fn test_insert_before_consolidate_text() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha</doc>"#).unwrap();
    let el_id = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    let txt = xot.new_text("Beta");
    xot.insert_before(el_id, txt).unwrap();
    assert_eq!(xot.text(el_id).map(|n| n.get()), Some("BetaAlpha"));
    assert_eq!(xot.serialize_to_string(doc), r#"<doc>BetaAlpha</doc>"#);
}

#[test]
fn test_insert_after_consolidate_text() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha</doc>"#).unwrap();
    let el_id = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    let txt = xot.new_text("Beta");
    xot.insert_after(el_id, txt).unwrap();
    assert_eq!(xot.text(el_id).map(|n| n.get()), Some("AlphaBeta"));
    assert_eq!(xot.serialize_to_string(doc), r#"<doc>AlphaBeta</doc>"#);
}

#[test]
fn test_prepend_consolidate_text() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha</doc>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    let txt = xot.new_text("Beta");
    xot.prepend(el_id, txt).unwrap();
    let text_el_id = xot.first_child(el_id).unwrap();
    assert_eq!(xot.text(text_el_id).map(|n| n.get()), Some("BetaAlpha"));
    assert_eq!(xot.serialize_to_string(doc), r#"<doc>BetaAlpha</doc>"#);
}

#[test]
fn test_root_node_can_have_only_single_element_append() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc/>"#).unwrap();
    let name = xot.add_name("a");
    assert!(xot.append_element(doc, name).is_err());
}

#[test]
fn test_root_node_can_have_only_single_element_insert_before() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc/>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    let name = xot.add_name("a");
    let new_el_id = xot.new_element(name);
    assert!(xot.insert_before(el_id, new_el_id).is_err());
}

#[test]
fn test_root_node_append_comment() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc/>"#).unwrap();
    xot.append_comment(doc, "hello").unwrap();
    assert_eq!(xot.serialize_to_string(doc), r#"<doc/><!--hello-->"#);
}

#[test]
fn test_remove_text_consolidation() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha<a/>Beta</doc>"#).unwrap();
    let el_id = xot
        .children(xot.document_element(doc).unwrap())
        .nth(1)
        .unwrap();
    // we found the a element
    let a = xot.name("a").unwrap();
    assert_eq!(xot.element(el_id).unwrap().name_id(), a);
    // now we remove it
    xot.remove(el_id).unwrap();
    // we should have a single text node
    let text_el_id = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    assert_eq!(xot.text_str(text_el_id), Some("AlphaBeta"));
    assert_eq!(xot.serialize_to_string(doc), r#"<doc>AlphaBeta</doc>"#);
}

#[test]
fn test_move_text_consolidation() {
    let mut xot = Xot::new();
    let doc_a = xot.parse(r#"<doc></doc>"#).unwrap();
    let doc_b = xot.parse(r#"<doc>Alpha<a/>Beta</doc>"#).unwrap();

    let a_id = xot
        .children(xot.document_element(doc_b).unwrap())
        .nth(1)
        .unwrap();
    // we found the a element
    let a = xot.name("a").unwrap();
    assert_eq!(xot.element(a_id).unwrap().name_id(), a);
    // now we append it into doc_a
    let doc_a_root = xot.document_element(doc_a).unwrap();
    xot.append(doc_a_root, a_id).unwrap();
    // we should have a single text node in b
    let text_el_id = xot
        .first_child(xot.document_element(doc_b).unwrap())
        .unwrap();
    assert_eq!(xot.text_str(text_el_id), Some("AlphaBeta"));
    assert_eq!(xot.serialize_to_string(doc_a), r#"<doc><a/></doc>"#);
    assert_eq!(xot.serialize_to_string(doc_b), r#"<doc>AlphaBeta</doc>"#);
}

#[test]
fn test_move_not_allowed_as_takes_document_element() {
    let mut xot = Xot::new();
    let doc_a = xot.parse(r#"<doc></doc>"#).unwrap();
    let doc_b = xot.parse(r#"<doc></doc>"#).unwrap();

    let el_a = xot.document_element(doc_a).unwrap();
    let el_b = xot.document_element(doc_b).unwrap();

    // not allowed as el_b is document element
    assert!(xot.append(el_a, el_b).is_err());
}

#[test]
fn test_create_missing_prefixes() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc></doc>"#).unwrap();
    let root_id = xot.document_element(doc).unwrap();
    let ns_id = xot.add_namespace("http://example.com");
    let name_id = xot.add_name_ns("a", ns_id);
    xot.append_element(root_id, name_id).unwrap();
    xot.create_missing_prefixes(root_id).unwrap();
    assert_eq!(
        xot.serialize_to_string(doc),
        r#"<doc xmlns:n0="http://example.com"><n0:a/></doc>"#
    );
}

#[test]
fn test_clone() {
    let mut xot = Xot::new();
    let root = xot.parse(r#"<doc><a>Hello!</a></doc>"#).unwrap();
    let doc_id = xot.document_element(root).unwrap();
    let a_id = xot.first_child(doc_id).unwrap();
    let a_id_clone = xot.clone(a_id);
    // change original won't affect the clone
    xot.text_mut(xot.first_child(a_id).unwrap())
        .unwrap()
        .set("Goodbye!");
    assert_eq!(
        xot.serialize_to_string(root),
        r#"<doc><a>Goodbye!</a></doc>"#
    );
    assert!(!xot.is_removed(a_id_clone));
    assert_eq!(
        xot.serialize_fragment_to_string(a_id_clone),
        r#"<a>Hello!</a>"#
    );
}

#[test]
fn test_clone_namespace() {
    let mut xot = Xot::new();
    let root = xot
        .parse(r#"<doc xmlns="http://example.com"><a>Hello!</a></doc>"#)
        .unwrap();
    let doc_id = xot.document_element(root).unwrap();
    let a_id = xot.first_child(doc_id).unwrap();
    let a_id_clone = xot.clone(a_id);
    // change original won't affect the clone
    xot.text_mut(xot.first_child(a_id).unwrap())
        .unwrap()
        .set("Goodbye!");
    assert_eq!(
        xot.serialize_to_string(root),
        r#"<doc xmlns="http://example.com"><a>Goodbye!</a></doc>"#
    );
    assert!(!xot.is_removed(a_id_clone));
    assert_eq!(
        xot.serialize_fragment_to_string(a_id_clone),
        r#"<a xmlns="http://example.com">Hello!</a>"#
    );
}
