use xot::{Error, Value, Xot};

#[test]
fn test_manipulate_text() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Data</doc>"#).unwrap();
    let text_id = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    if let Value::Text(node) = xot.value_mut(text_id) {
        node.set("Changed");
    }
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>Changed</doc>"#);
}

#[test]
fn test_manipulate_attribute() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc a="Foo"/>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    let a = xot.name("a").unwrap();

    let mut attributes = xot.attributes_mut(el_id);
    attributes.insert(a, "Changed".to_string());

    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc a="Changed"/>"#);
}

#[test]
fn test_add_attribute() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc/>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    assert!(xot.name("a").is_none());
    let a = xot.add_name("a");

    let mut attributes = xot.attributes_mut(el_id);
    attributes.insert(a, "Created".to_string());

    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc a="Created"/>"#);
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

    let mut attributes = xot.attributes_mut(el_id);
    attributes.insert(a, "Changed".to_string());

    assert_eq!(
        xot.to_string(doc).unwrap(),
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

    let mut attributes = xot.attributes_mut(el_id);
    attributes.insert(a, "Created".to_string());
    assert_eq!(
        xot.to_string(doc).unwrap(),
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
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc><a/></doc>"#);
}

#[test]
fn test_prepend_element() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc><b/></doc>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    let name = xot.add_name("a");
    let new_el_id = xot.new_element(name);
    xot.prepend(el_id, new_el_id).unwrap();
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc><a/><b/></doc>"#);
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
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc><a/><b/></doc>"#);
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
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc><b/><a/></doc>"#);
}

#[test]
fn test_append_text() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc/>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    xot.append_text(el_id, "Changed").unwrap();
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>Changed</doc>"#);
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
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>AlphaBeta</doc>"#);
}

#[test]
fn test_append_text_after_text_no_consolidates_nodes() {
    let mut xot = Xot::new();
    xot.set_text_consolidation(false);
    let doc = xot.parse(r#"<doc/>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    xot.append_text(el_id, "Alpha").unwrap();
    xot.append_text(el_id, "Beta").unwrap();
    let mut children = xot.children(el_id);
    assert_eq!(xot.text_str(children.next().unwrap()).unwrap(), "Alpha");
    assert_eq!(xot.text_str(children.next().unwrap()).unwrap(), "Beta");
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>AlphaBeta</doc>"#);
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
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>AlphaBeta</doc>"#);
}

#[test]
fn test_append_text_after_text_no_consolidates_nodes_direct_append() {
    let mut xot = Xot::new();
    xot.set_text_consolidation(false);
    let doc = xot.parse(r#"<doc/>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    let txt1 = xot.new_text("Alpha");
    let txt2 = xot.new_text("Beta");
    xot.append(el_id, txt1).unwrap();
    xot.append(el_id, txt2).unwrap();
    let mut children = xot.children(el_id);
    assert_eq!(xot.text_str(children.next().unwrap()).unwrap(), "Alpha");
    assert_eq!(xot.text_str(children.next().unwrap()).unwrap(), "Beta");
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>AlphaBeta</doc>"#);
}

#[test]
fn test_insert_before_consolidate_text() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha</doc>"#).unwrap();
    let el_id = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    let txt = xot.new_text("Beta");
    xot.insert_before(el_id, txt).unwrap();
    assert_eq!(xot.text(el_id).map(|n| n.get()), Some("BetaAlpha"));
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>BetaAlpha</doc>"#);
}

#[test]
fn test_insert_after_consolidate_text() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha</doc>"#).unwrap();
    let el_id = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    let txt = xot.new_text("Beta");
    xot.insert_after(el_id, txt).unwrap();
    assert_eq!(xot.text(el_id).map(|n| n.get()), Some("AlphaBeta"));
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>AlphaBeta</doc>"#);
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
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>BetaAlpha</doc>"#);
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
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc/><!--hello-->"#);
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
    assert_eq!(xot.element(el_id).unwrap().name(), a);
    // now we remove it
    xot.remove(el_id).unwrap();
    // we should have a single text node
    let text_el_id = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    assert_eq!(xot.text_str(text_el_id), Some("AlphaBeta"));
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>AlphaBeta</doc>"#);
}

#[test]
fn test_remove_text_no_consolidation() {
    let mut xot = Xot::new();
    xot.set_text_consolidation(false);
    let doc = xot.parse(r#"<doc>Alpha<a/>Beta</doc>"#).unwrap();
    let el_id = xot
        .children(xot.document_element(doc).unwrap())
        .nth(1)
        .unwrap();
    // we found the a element
    let a = xot.name("a").unwrap();
    assert_eq!(xot.element(el_id).unwrap().name(), a);
    // now we remove it
    xot.remove(el_id).unwrap();
    // we have two children
    let mut children = xot.children(xot.document_element(doc).unwrap());
    assert_eq!(xot.text_str(children.next().unwrap()).unwrap(), "Alpha");
    assert_eq!(xot.text_str(children.next().unwrap()).unwrap(), "Beta");
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>AlphaBeta</doc>"#);
}

#[test]
fn test_remove_root() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha<a/>Beta</doc>"#).unwrap();
    xot.remove(doc).unwrap();
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
    assert_eq!(xot.element(a_id).unwrap().name(), a);
    // now we append it into doc_a
    let doc_a_root = xot.document_element(doc_a).unwrap();
    xot.append(doc_a_root, a_id).unwrap();
    // we should have a single text node in b
    let text_el_id = xot
        .first_child(xot.document_element(doc_b).unwrap())
        .unwrap();
    assert_eq!(xot.text_str(text_el_id), Some("AlphaBeta"));
    assert_eq!(xot.to_string(doc_a).unwrap(), r#"<doc><a/></doc>"#);
    assert_eq!(xot.to_string(doc_b).unwrap(), r#"<doc>AlphaBeta</doc>"#);
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
        xot.to_string(root).unwrap(),
        r#"<doc><a>Goodbye!</a></doc>"#
    );
    assert!(!xot.is_removed(a_id_clone));
    assert_eq!(xot.to_string(a_id_clone).unwrap(), r#"<a>Hello!</a>"#);
}

#[test]
fn test_clone_root() {
    let mut xot = Xot::new();
    let root = xot.parse(r#"<doc><a>Hello!</a></doc>"#).unwrap();
    let root_clone = xot.clone(root);
    assert_eq!(
        xot.to_string(root_clone).unwrap(),
        r#"<doc><a>Hello!</a></doc>"#
    );
}

#[test]
fn test_clone_root_after_insert() {
    let mut xot = Xot::new();
    let root = xot.parse("<doc>hello <i>world</i>!</doc>").unwrap();
    let doc = xot.document_element(root).unwrap();
    let txt = xot.first_child(doc).unwrap();
    let i = xot.next_sibling(txt).unwrap();
    let new_node = xot.new_text("?");
    xot.insert_after(i, new_node).unwrap();
    let root_clone = xot.clone(root);
    assert_eq!(
        xot.to_string(root_clone).unwrap(),
        "<doc>hello <i>world</i>?!</doc>"
    );
}

#[test]
fn test_clone_root_after_insert_no_consolidation() {
    let mut xot = Xot::new();
    xot.set_text_consolidation(false);
    let root = xot.parse("<doc>hello <i>world</i>!</doc>").unwrap();
    let doc = xot.document_element(root).unwrap();
    let txt = xot.first_child(doc).unwrap();
    let i = xot.next_sibling(txt).unwrap();
    let new_node = xot.new_text("?");
    xot.insert_after(i, new_node).unwrap();
    let root_clone = xot.clone(root);
    xot.set_text_consolidation(true);
    assert_eq!(
        xot.to_string(root_clone).unwrap(),
        "<doc>hello <i>world</i>?!</doc>"
    );
}

#[test]
fn test_clone_root_after_insert_no_consolidation_for_insert_consolidation_for_clone() {
    let mut xot = Xot::new();
    xot.set_text_consolidation(false);
    let root = xot.parse("<doc>hello <i>world</i>!</doc>").unwrap();
    let doc = xot.document_element(root).unwrap();
    let txt = xot.first_child(doc).unwrap();
    let i = xot.next_sibling(txt).unwrap();
    let new_node = xot.new_text("?");
    xot.insert_after(i, new_node).unwrap();
    xot.set_text_consolidation(true);
    let root_clone = xot.clone(root);
    assert_eq!(
        xot.to_string(root_clone).unwrap(),
        "<doc>hello <i>world</i>?!</doc>"
    );
}

#[test]
fn test_insert_after_consolidation() {
    let mut xot = Xot::new();
    let root = xot.parse("<doc>hello <i>world</i>!</doc>").unwrap();
    let doc = xot.document_element(root).unwrap();
    let txt = xot.first_child(doc).unwrap();
    let i = xot.next_sibling(txt).unwrap();
    let new_node = xot.new_text("?");
    xot.insert_after(i, new_node).unwrap();
    assert_eq!(xot.children(doc).count(), 3);
}

#[test]
fn test_clone_with_namespaces() {
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
        xot.to_string(root).unwrap(),
        r#"<doc xmlns="http://example.com"><a>Goodbye!</a></doc>"#
    );
    assert!(!xot.is_removed(a_id_clone));
    xot.create_missing_prefixes(a_id_clone).unwrap();
    assert_eq!(
        xot.to_string(a_id_clone).unwrap(),
        r#"<n0:a xmlns:n0="http://example.com">Hello!</n0:a>"#
    );
}

#[test]
fn test_clone_with_prefixes() {
    let mut xot = Xot::new();
    let root = xot
        .parse(r#"<doc xmlns="http://example.com"><a>Hello!</a></doc>"#)
        .unwrap();
    let doc_id = xot.document_element(root).unwrap();
    let a_id = xot.first_child(doc_id).unwrap();
    let a_id_clone = xot.clone_with_prefixes(a_id);
    // change original won't affect the clone
    xot.text_mut(xot.first_child(a_id).unwrap())
        .unwrap()
        .set("Goodbye!");
    assert_eq!(
        xot.to_string(root).unwrap(),
        r#"<doc xmlns="http://example.com"><a>Goodbye!</a></doc>"#
    );
    assert!(!xot.is_removed(a_id_clone));
    assert_eq!(
        xot.to_string(a_id_clone).unwrap(),
        r#"<a xmlns="http://example.com">Hello!</a>"#
    );
}

#[test]
fn test_clone_with_prefixes_only_necessary_ones() {
    let mut xot = Xot::new();
    let root = xot
        .parse(r#"<doc xmlns:a="http://example.com/a" xmlns:b="http://example.com/b"><a:p>Hello!</a:p></doc>"#)
        .unwrap();
    let doc_id = xot.document_element(root).unwrap();
    let a_id = xot.first_child(doc_id).unwrap();
    let a_id_clone = xot.clone_with_prefixes(a_id);
    assert_eq!(
        xot.to_string(a_id_clone).unwrap(),
        r#"<a:p xmlns:a="http://example.com/a">Hello!</a:p>"#
    );
}

#[test]
fn test_element_unwrap() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc><first/><a><one/><two/></a><second/></doc>"#)
        .unwrap();
    let el_id = xot
        .children(xot.document_element(doc).unwrap())
        .nth(1)
        .unwrap();
    // we found the a element
    let a = xot.name("a").unwrap();
    assert_eq!(xot.element(el_id).unwrap().name(), a);
    // now we unwrap it
    xot.element_unwrap(el_id).unwrap();

    assert_eq!(
        xot.to_string(doc).unwrap(),
        r#"<doc><first/><one/><two/><second/></doc>"#
    );
}

#[test]
fn test_element_unwrap_consolidation_single_element() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha<a/>Beta</doc>"#).unwrap();
    let el_id = xot
        .children(xot.document_element(doc).unwrap())
        .nth(1)
        .unwrap();
    // we found the a element
    let a = xot.name("a").unwrap();
    assert_eq!(xot.element(el_id).unwrap().name(), a);
    // now we unwrap it
    xot.element_unwrap(el_id).unwrap();
    // we should have a single text node
    let text_el_id = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    assert_eq!(xot.text_str(text_el_id), Some("AlphaBeta"));
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>AlphaBeta</doc>"#);
}

#[test]
fn test_element_unwrap_consolidation_text_in_element() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha<a>!</a>Beta</doc>"#).unwrap();
    let el_id = xot
        .children(xot.document_element(doc).unwrap())
        .nth(1)
        .unwrap();
    // we found the a element
    let a = xot.name("a").unwrap();
    assert_eq!(xot.element(el_id).unwrap().name(), a);
    // now we unwrap it
    xot.element_unwrap(el_id).unwrap();
    // we should have a single text node
    let text_el_id = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    assert_eq!(xot.text_str(text_el_id), Some("Alpha!Beta"));
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>Alpha!Beta</doc>"#);
}

#[test]
fn test_element_unwrap_consolidation_text_in_element_at_beginning() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha<a>!<b/></a>Beta</doc>"#).unwrap();
    let el_id = xot
        .children(xot.document_element(doc).unwrap())
        .nth(1)
        .unwrap();
    // we found the a element
    let a = xot.name("a").unwrap();
    assert_eq!(xot.element(el_id).unwrap().name(), a);
    // now we unwrap it
    xot.element_unwrap(el_id).unwrap();
    let text_el_id = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    assert_eq!(xot.text_str(text_el_id), Some("Alpha!"));
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>Alpha!<b/>Beta</doc>"#);
}

#[test]
fn test_element_unwrap_consolidation_text_in_element_at_end() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha<a><b/>!</a>Beta</doc>"#).unwrap();
    let el_id = xot
        .children(xot.document_element(doc).unwrap())
        .nth(1)
        .unwrap();
    // we found the a element
    let a = xot.name("a").unwrap();
    assert_eq!(xot.element(el_id).unwrap().name(), a);
    // now we unwrap it
    xot.element_unwrap(el_id).unwrap();

    let text_el_id = xot.last_child(xot.document_element(doc).unwrap()).unwrap();
    assert_eq!(xot.text_str(text_el_id), Some("!Beta"));
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>Alpha<b/>!Beta</doc>"#);
}

#[test]
fn test_element_unwrap_consolidation_text_in_element_both_ends() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha<a>?<b/>!</a>Beta</doc>"#).unwrap();
    let el_id = xot
        .children(xot.document_element(doc).unwrap())
        .nth(1)
        .unwrap();
    // we found the a element
    let a = xot.name("a").unwrap();
    assert_eq!(xot.element(el_id).unwrap().name(), a);
    // now we unwrap it
    xot.element_unwrap(el_id).unwrap();

    let text_el_id = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    assert_eq!(xot.text_str(text_el_id), Some("Alpha?"));
    let text_el_id = xot.last_child(xot.document_element(doc).unwrap()).unwrap();
    assert_eq!(xot.text_str(text_el_id), Some("!Beta"));
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>Alpha?<b/>!Beta</doc>"#);
}

#[test]
fn test_element_unwrap_document_element() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc><p>Hello</p></doc>"#).unwrap();
    let document_element = xot.document_element(doc).unwrap();

    xot.element_unwrap(document_element).unwrap();

    assert_eq!(xot.to_string(doc).unwrap(), r#"<p>Hello</p>"#);
}

#[test]
fn test_element_unwrap_document_element_not_allowed_multiple_children() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc><a/><b/></doc>"#).unwrap();
    let document_element = xot.document_element(doc).unwrap();

    assert!(xot.element_unwrap(document_element).is_err());
}

#[test]
fn test_element_unwrap_document_element_not_allowed_non_element_child() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Text</doc>"#).unwrap();
    let document_element = xot.document_element(doc).unwrap();

    assert!(xot.element_unwrap(document_element).is_err());
}

#[test]
fn test_element_wrap() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha</doc>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let txt_el = xot.first_child(doc_el).unwrap();
    let name_p = xot.add_name("p");
    xot.element_wrap(txt_el, name_p).unwrap();
    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc><p>Alpha</p></doc>"#);
}

#[test]
fn test_element_wrap_middle() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc><first/>Alpha<second/></doc>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let txt_el = xot.children(doc_el).nth(1).unwrap();
    let name_p = xot.add_name("p");
    xot.element_wrap(txt_el, name_p).unwrap();
    assert_eq!(
        xot.to_string(doc).unwrap(),
        r#"<doc><first/><p>Alpha</p><second/></doc>"#
    );
}

#[test]
fn test_element_wrap_document_element() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha</doc>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let name_p = xot.add_name("p");
    xot.element_wrap(doc_el, name_p).unwrap();
    assert_eq!(xot.to_string(doc).unwrap(), r#"<p><doc>Alpha</doc></p>"#);
}

#[test]
fn test_element_wrap_element_under_root_not_document_element() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<!-- hello --><doc>Alpha</doc>"#).unwrap();
    let comment_el = xot.first_child(doc).unwrap();
    let name_p = xot.add_name("p");
    assert!(xot.element_wrap(comment_el, name_p).is_err());
}

#[test]
fn test_element_wrap_standalone_element() {
    let mut xot = Xot::new();
    let element_name = xot.add_name("element");
    let element = xot.new_element(element_name);
    let name_p = xot.add_name("p");
    let wrapper = xot.element_wrap(element, name_p).unwrap();
    assert_eq!(xot.to_string(wrapper).unwrap(), r#"<p><element/></p>"#);
}

#[test]
fn test_replace_node() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha</doc>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let replaced = xot.first_child(doc_el).unwrap();

    let name_p = xot.add_name("p");
    let replacing = xot.new_element(name_p);

    xot.replace(replaced, replacing).unwrap();

    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc><p/></doc>"#);
}

#[test]
fn test_replace_document_element() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha</doc>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();

    let name_p = xot.add_name("p");
    let replacing = xot.new_element(name_p);

    xot.replace(doc_el, replacing).unwrap();

    assert_eq!(xot.to_string(doc).unwrap(), r#"<p/>"#);
}

#[test]
fn test_replace_document_element_illegal() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha</doc>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();

    let replacing = xot.new_text("Sneaky");

    // you shouldn't be allowed to replace the document element with a text node, only
    // with an element node
    assert!(xot.replace(doc_el, replacing).is_err());
}

#[test]
fn test_detach() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc><a><b/></a></doc>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let detached = xot.first_child(doc_el).unwrap();

    xot.detach(detached).unwrap();

    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc/>"#);
    assert_eq!(xot.to_string(detached).unwrap(), r#"<a><b/></a>"#);
}

#[test]
fn test_replace_node_reconciliate_text_before() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc>Alpha<x/></doc>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let replaced = xot.children(doc_el).nth(1).unwrap();

    let replacing = xot.new_text("X");

    xot.replace(replaced, replacing).unwrap();

    let found = xot.first_child(doc_el).unwrap();
    assert_eq!(xot.text_str(found), Some("AlphaX"));

    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>AlphaX</doc>"#);
}

#[test]
fn test_replace_node_reconciliate_text_after() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc><x/>Alpha</doc>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let replaced = xot.first_child(doc_el).unwrap();

    let replacing = xot.new_text("X");

    xot.replace(replaced, replacing).unwrap();

    let found = xot.first_child(doc_el).unwrap();
    assert_eq!(xot.text_str(found), Some("XAlpha"));

    assert_eq!(xot.to_string(doc).unwrap(), r#"<doc>XAlpha</doc>"#);
}

#[test]
fn test_replace_node_reconciliates_where_detached() {
    let mut xot = Xot::new();
    let doc_a = xot.parse(r#"<doc><x/></doc>"#).unwrap();
    let doc_a_el = xot.document_element(doc_a).unwrap();
    let replaced = xot.first_child(doc_a_el).unwrap();

    let doc_b = xot.parse(r#"<doc>a<y/>b</doc>"#).unwrap();
    let doc_b_el = xot.document_element(doc_b).unwrap();
    let replacing = xot.children(doc_b_el).nth(1).unwrap();

    xot.replace(replaced, replacing).unwrap();

    let found = xot.first_child(doc_b_el).unwrap();
    assert_eq!(xot.text_str(found), Some("ab"));

    assert_eq!(xot.to_string(doc_a).unwrap(), r#"<doc><y/></doc>"#);
    assert_eq!(xot.to_string(doc_b).unwrap(), r#"<doc>ab</doc>"#);
}

#[test]
fn test_new_root() -> Result<(), Error> {
    let mut xot = Xot::new();
    let name = xot.add_name("doc");
    let doc_el = xot.new_element(name);

    let root = xot.new_root(doc_el)?;

    assert_eq!(xot.to_string(root).unwrap(), r#"<doc/>"#);
    Ok(())
}
