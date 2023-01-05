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

#[test]
fn test_prefix_ambiguous_no_ns() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(
            r#"<a><a xmlns:x="http://example.com/y"><a xmlns:x="http://example.com/x"/></a></a>"#,
        )
        .unwrap();
    let root_id = xot.document_element(doc).unwrap();
    let a_id = xot.first_child(root_id).unwrap();
    let a_id = xot.first_child(a_id).unwrap();

    let ns_y_id = xot.add_namespace("http://example.com/y");
    let name_q_id = xot.add_name_ns("q", ns_y_id);
    let element = xot.element_mut(a_id).unwrap();
    element.set_attribute(name_q_id, "Q");
    assert_eq!(
        xot.serialize_to_string(doc),
        r#"<a xmlns:n0="http://example.com/y"><a xmlns:x="http://example.com/y"><a xmlns:x="http://example.com/x" n0:q="Q"/></a></a>"#
    );
}

#[test]
fn test_prefix_ambiguous_default_ns() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<a xmlns="http://example.com/y"><a><a/></a></a>"#)
        .unwrap();
    let root_id = xot.document_element(doc).unwrap();
    let a_id = xot.first_child(root_id).unwrap();
    let a_id = xot.first_child(a_id).unwrap();

    let ns_y_id = xot.add_namespace("http://example.com/y");
    let ns_empty_id = xot.add_namespace("");

    let name_r_id = xot.add_name_ns("r", ns_y_id);
    let name_empty_id = xot.add_name_ns("r", ns_empty_id);
    let element = xot.element_mut(a_id).unwrap();
    element.set_attribute(name_r_id, "R");
    element.set_attribute(name_empty_id, "R2");
    assert_eq!(
        xot.serialize_to_string(doc),
        r#"<a xmlns="http://example.com/y" xmlns:n0="http://example.com/y"><a><a n0:r="R" r="R2"/></a></a>"#
    );
}

#[test]
fn test_prefix_ambiguous_default_ns2() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<a><a><a xmlns="http://example.com/y"/></a></a>"#)
        .unwrap();
    let root_id = xot.document_element(doc).unwrap();
    let a_id = xot.first_child(root_id).unwrap();
    let a_id = xot.first_child(a_id).unwrap();

    let ns_y_id = xot.add_namespace("http://example.com/y");

    let name_r_id = xot.add_name_ns("r", ns_y_id);
    let element = xot.element_mut(a_id).unwrap();
    element.set_attribute(name_r_id, "R");
    assert_eq!(
        xot.serialize_to_string(doc),
        r#"<a xmlns:n0="http://example.com/y"><a><a xmlns="http://example.com/y" n0:r="R"/></a></a>"#
    );
}
