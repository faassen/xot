use xot::Xot;

#[test]
fn test_serialize_node() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc xmlns:foo="http://example.com"><foo:a/></doc>"#)
        .unwrap();
    let node = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    assert_eq!(
        xot.serialize_node_to_string(node),
        r#"<foo:a xmlns:foo="http://example.com"/>"#
    );
}

#[test]
fn test_serialize_node_default_ns() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc xmlns="http://example.com"><a/></doc>"#)
        .unwrap();
    let node = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    assert_eq!(
        xot.serialize_node_to_string(node),
        r#"<a xmlns="http://example.com"/>"#
    );
}

#[test]
fn test_serialize_node_default_ns_nested() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc xmlns="http://example.com"><a><b/></a></doc>"#)
        .unwrap();
    let node = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    assert_eq!(
        xot.serialize_node_to_string(node),
        r#"<a xmlns="http://example.com"><b/></a>"#
    );
}

#[test]
fn test_prefix_ambiguous() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc xmlns:x="http://example.com/x"><a xmlns:x="http://example.com/y"/></doc>"#)
        .unwrap();
    let root_id = xot.document_element(doc).unwrap();
    let a_id = xot.first_child(root_id).unwrap();

    let ns_x_id = xot.add_namespace("http://example.com/x");
    let ns_y_id = xot.add_namespace("http://example.com/y");
    let name_x_id = xot.add_name_ns("b", ns_x_id);
    let name_y_id = xot.add_name_ns("b", ns_y_id);

    let element = xot.element_mut(a_id).unwrap();
    element.set_attribute(name_x_id, "X");
    element.set_attribute(name_y_id, "Y");
    assert_eq!(
        xot.serialize_to_string(doc),
        r#"<doc xmlns:x="http://example.com/x" xmlns:n0="http://example.com/x"><a xmlns:x="http://example.com/y" n0:b="X" x:b="Y"/></doc>"#
    );
}
