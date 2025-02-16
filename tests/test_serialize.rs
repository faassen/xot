use xot::Xot;

#[test]
fn test_serialize_node() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc xmlns:foo="http://example.com"><foo:a/></doc>"#)
        .unwrap();
    let node = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    assert_eq!(
        xot.to_string(node).unwrap(),
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
        xot.to_string(node).unwrap(),
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
        xot.to_string(node).unwrap(),
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

    let mut attributes = xot.attributes_mut(a_id);
    attributes.insert(name_x_id, "X".to_string());
    attributes.insert(name_y_id, "Y".to_string());

    xot.create_missing_prefixes(doc).unwrap();

    assert_eq!(
        xot.to_string(doc).unwrap(),
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

    let mut attributes = xot.attributes_mut(a_id);
    attributes.insert(name_q_id, "Q".to_string());

    xot.create_missing_prefixes(doc).unwrap();
    assert_eq!(
        xot.to_string(doc).unwrap(),
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

    let mut attributes = xot.attributes_mut(a_id);
    attributes.insert(name_r_id, "R".to_string());
    attributes.insert(name_empty_id, "R2".to_string());
    xot.create_missing_prefixes(doc).unwrap();
    assert_eq!(
        xot.to_string(doc).unwrap(),
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
    let mut attributes = xot.attributes_mut(a_id);
    attributes.insert(name_r_id, "R".to_string());

    xot.create_missing_prefixes(doc).unwrap();
    assert_eq!(
        xot.to_string(doc).unwrap(),
        r#"<a xmlns:n0="http://example.com/y"><a><a xmlns="http://example.com/y" n0:r="R"/></a></a>"#
    );
}

#[test]
fn test_serialize_lt() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a>&lt;</a>"#).unwrap();
    let serialized = xot.to_string(doc).unwrap();
    assert_eq!(serialized, r#"<a>&lt;</a>"#);
}

#[test]
fn test_serialize_gt_by_default() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a>&gt;</a>"#).unwrap();
    let serialized = xot.to_string(doc).unwrap();
    assert_eq!(serialized, r#"<a>&gt;</a>"#);
}

#[test]
fn test_do_not_serialize_gt() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a>&gt;</a>"#).unwrap();
    let serialized = xot
        .serialize_xml_string(
            xot::output::xml::Parameters {
                unescaped_gt: true,
                ..Default::default()
            },
            doc,
        )
        .unwrap();
    assert_eq!(serialized, r#"<a>></a>"#);
}

#[test]
fn test_weird_delimiter() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a>]]&gt;</a>"#).unwrap();
    let serialized = xot.to_string(doc).unwrap();
    assert_eq!(serialized, r#"<a>]]&gt;</a>"#);
}

#[test]
fn test_weird_delimiter_unescaped() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a>]]&gt;</a>"#).unwrap();
    let serialized = xot
        .serialize_xml_string(
            xot::output::xml::Parameters {
                unescaped_gt: true,
                ..Default::default()
            },
            doc,
        )
        .unwrap();
    assert_eq!(serialized, r#"<a>]]&gt;</a>"#);
}

#[test]
fn test_serialize_fragment() {
    let mut xot = Xot::new();
    let fragment = xot.parse_fragment(r#"<a/><b/>text"#).unwrap();
    let serialized = xot.to_string(fragment).unwrap();
    assert_eq!(serialized, r#"<a/><b/>text"#);
}

#[test]
fn test_serialize_empty_fragment() {
    let mut xot = Xot::new();
    let fragment = xot.parse_fragment(r#""#).unwrap();
    let serialized = xot.to_string(fragment).unwrap();
    assert_eq!(serialized, r#""#);
}

#[test]
fn test_serialize_text_fragment() {
    let mut xot = Xot::new();
    let fragment = xot.parse_fragment(r#"text"#).unwrap();
    let serialized = xot.to_string(fragment).unwrap();
    assert_eq!(serialized, r#"text"#);
}
