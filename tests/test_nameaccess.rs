use xot::Xot;

#[test]
fn test_deduplicate_namespace() {
    let mut xot = Xot::new();
    let root = xot
        .parse(r#"<doc xmlns="http://example.com"><a xmlns="http://example.com">Hello!</a></doc>"#)
        .unwrap();
    xot.deduplicate_namespaces(root);
    assert_eq!(
        xot.to_string(root).unwrap(),
        r#"<doc xmlns="http://example.com"><a>Hello!</a></doc>"#
    );
}

#[test]
fn test_deduplicate_named_namespace() {
    let mut xot = Xot::new();
    let root = xot
        .parse(r#"<doc xmlns="http://example.com"><foo:a xmlns:foo="http://example.com">Hello!</foo:a></doc>"#)
        .unwrap();
    xot.deduplicate_namespaces(root);
    assert_eq!(
        xot.to_string(root).unwrap(),
        r#"<doc xmlns="http://example.com"><a>Hello!</a></doc>"#
    );
}

#[test]
fn test_deduplicate_named_namespace_again() {
    let mut xot = Xot::new();
    let root = xot
        .parse(r#"<section xmlns="http://docbook.org/ns/docbook" xmlns:diff="http://paligo.net/nxd" version="5.0">
  <title>Title</title>
  <para diff:delete="">Para first old </para><para xmlns="http://docbook.org/ns/docbook" diff:insert="">Before emphasis <emphasis>emphasis</emphasis> After emphasis</para>
  <para diff:delete="">Para second old</para><warning xmlns="http://docbook.org/ns/docbook" diff:insert="">
    <title>I am new</title>
    <para>Warning here</para>
  </warning>
</section>"#)
        .unwrap();
    xot.deduplicate_namespaces(root);
    assert_eq!(
        xot.to_string(root).unwrap(),
        r#"<section xmlns="http://docbook.org/ns/docbook" xmlns:diff="http://paligo.net/nxd" version="5.0">
  <title>Title</title>
  <para diff:delete="">Para first old </para><para diff:insert="">Before emphasis <emphasis>emphasis</emphasis> After emphasis</para>
  <para diff:delete="">Para second old</para><warning diff:insert="">
    <title>I am new</title>
    <para>Warning here</para>
  </warning>
</section>"#
    );
}

#[test]
fn test_deduplicate_overlapping_namespace_attribute() {
    let mut xot = Xot::new();

    // we have a default namespace and an attribute in that namespace
    // deduplicate should not clean up the overlapping n prefix in this
    // case as it's in use by the attribute.
    let doc = xot
        .parse(r#"<a xmlns="N"><b xmlns:n="N" n:c="C"/></a>"#)
        .unwrap();

    xot.deduplicate_namespaces(doc);

    // now serialize the doc; we haven't removed prefix n as it's in use by an
    // attribute
    assert_eq!(
        xot.to_string(doc).unwrap(),
        r#"<a xmlns="N"><b xmlns:n="N" n:c="C"/></a>"#
    );
}

#[test]
fn test_deduplicate_overlapping_namespace_element() {
    let mut xot = Xot::new();

    let doc = xot
        .parse(r#"<a xmlns="N"><b xmlns:n="N"><n:c/></b></a>"#)
        .unwrap();

    xot.deduplicate_namespaces(doc);

    // now serialize the doc; we have removed prefix n as it's in use by
    // an element only
    assert_eq!(
        xot.to_string(doc).unwrap(),
        r#"<a xmlns="N"><b><c/></b></a>"#
    );
}

#[test]
fn test_deduplicate_overlapping_namespace_deeper() {
    let mut xot = Xot::new();

    // we have a default namespace and an attribute in that namespace
    // deduplicate should not clean up the overlapping n prefix in this
    // case as it's in use by the attribute. Here we test the case
    // where this attribute is deeper down the tree, away from where the
    // prefix is initialized.
    let doc = xot
        .parse(r#"<a xmlns="N"><b xmlns:n="N"><sub n:c="C"/></b></a>"#)
        .unwrap();

    xot.deduplicate_namespaces(doc);

    // now serialize the doc
    // we haven't removed prefix n as it's in use by an attribute
    assert_eq!(
        xot.to_string(doc).unwrap(),
        r#"<a xmlns="N"><b xmlns:n="N"><sub n:c="C"/></b></a>"#
    );
}

#[test]
fn test_name_ns_str_no_namespace() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a/>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let name = xot.element(doc_el).unwrap().name();
    assert_eq!(xot.name_ns_str(name), ("a", ""));
}

#[test]
fn test_name_ns_str_namespace() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a xmlns="http://example.com" />"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let name = xot.element(doc_el).unwrap().name();
    assert_eq!(xot.name_ns_str(name), ("a", "http://example.com"));
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
        xot.to_string(doc).unwrap(),
        r#"<doc xmlns:n0="http://example.com"><n0:a/></doc>"#
    );
}

#[test]
fn test_unresolved_namespaces() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc xmlns:a="http://example.com/a"><a:p/></doc>"#)
        .unwrap();
    let root_id = xot.document_element(doc).unwrap();
    let p_el = xot.first_child(root_id).unwrap();
    let a_ns = xot.add_namespace("http://example.com/a");

    assert_eq!(xot.unresolved_namespaces(p_el), [a_ns]);
}

#[test]
fn test_unresolved_namespaces_resolved() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc xmlns:a="http://example.com/a"><a:p xmlns:a="http://example.com/b"/></doc>"#)
        .unwrap();
    let root_id = xot.document_element(doc).unwrap();
    let p_el = xot.first_child(root_id).unwrap();

    assert_eq!(xot.unresolved_namespaces(p_el), []);
}

#[test]
fn test_unresolved_namespaces_resolved_deeper() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc xmlns:a="http://example.com/a"><a:p><b:x xmlns:b="http://example.com/x"/></a:p></doc>"#)
        .unwrap();
    let root_id = xot.document_element(doc).unwrap();
    let p_el = xot.first_child(root_id).unwrap();
    let a_ns = xot.add_namespace("http://example.com/a");

    assert_eq!(xot.unresolved_namespaces(p_el), [a_ns]);
}

#[test]
fn test_is_prefix_defined() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc xmlns:a="http://example.com/a"><a:p/></doc>"#)
        .unwrap();
    let root_id = xot.document_element(doc).unwrap();
    let a_prefix = xot.add_prefix("a");
    let b_prefix = xot.add_prefix("b");
    let xml_prefix = xot.add_prefix("xml");
    let p0 = xot.first_child(root_id).unwrap();
    // let b_ns = xot.add_namespace("http://example.com/b");
    // let p1_name = xot.add_name_ns("p", b_ns);
    // xot.append_element(root_id, p1_name).unwrap();

    assert!(xot.is_prefix_defined(root_id, a_prefix));
    assert!(xot.is_prefix_defined(p0, a_prefix));
    assert!(!xot.is_prefix_defined(p0, b_prefix));
    assert!(xot.is_prefix_defined(root_id, xml_prefix));
}

#[test]
fn test_prefix_for_namespace() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc xmlns:a="http://example.com/a"><a:p/></doc>"#)
        .unwrap();
    let root_id = xot.document_element(doc).unwrap();
    let a_prefix = xot.add_prefix("a");
    let a_ns = xot.add_namespace("http://example.com/a");
    let b_ns = xot.add_namespace("http://example.com/b");
    let xml_prefix = xot.add_prefix("xml");
    let xml_ns = xot.add_namespace("http://www.w3.org/XML/1998/namespace");
    let p0 = xot.first_child(root_id).unwrap();
    // add a p1 that doesn't have a prefix
    let p1_name = xot.add_name_ns("p", b_ns);
    xot.append_element(root_id, p1_name).unwrap();
    let p1 = xot.next_sibling(p0).unwrap();

    assert_eq!(xot.prefix_for_namespace(root_id, a_ns), Some(a_prefix));
    assert_eq!(xot.prefix_for_namespace(p0, a_ns), Some(a_prefix));
    assert_eq!(xot.prefix_for_namespace(p0, b_ns), None);
    assert_eq!(xot.prefix_for_namespace(p1, b_ns), None);
    assert_eq!(xot.prefix_for_namespace(p0, xml_ns), Some(xml_prefix));
}

#[test]
fn test_namespace_for_prefix() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc xmlns:a="http://example.com/a"><a:p/></doc>"#)
        .unwrap();
    let root_id = xot.document_element(doc).unwrap();
    let a_prefix = xot.add_prefix("a");
    let a_ns = xot.add_namespace("http://example.com/a");
    let b_ns = xot.add_namespace("http://example.com/b");
    let b_prefix = xot.add_prefix("b");
    let xml_prefix = xot.add_prefix("xml");
    let xml_ns = xot.add_namespace("http://www.w3.org/XML/1998/namespace");
    let p0 = xot.first_child(root_id).unwrap();
    // add a p1 that doesn't have a prefix
    let p1_name = xot.add_name_ns("p", b_ns);
    xot.append_element(root_id, p1_name).unwrap();
    let p1 = xot.next_sibling(p0).unwrap();

    assert_eq!(xot.namespace_for_prefix(root_id, a_prefix), Some(a_ns));
    assert_eq!(xot.namespace_for_prefix(p0, a_prefix), Some(a_ns));
    assert_eq!(xot.namespace_for_prefix(p0, b_prefix), None);
    assert_eq!(xot.namespace_for_prefix(p1, b_prefix), None);
    assert_eq!(xot.namespace_for_prefix(p0, xml_prefix), Some(xml_ns));
}

#[test]
fn test_namespaces() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc xmlns:a="http://example.com/a"><a:p/></doc>"#)
        .unwrap();
    let root_id = xot.document_element(doc).unwrap();
    let a_prefix = xot.add_prefix("a");
    let a_ns = xot.add_namespace("http://example.com/a");
    let b_ns = xot.add_namespace("http://example.com/b");
    let xml_prefix = xot.add_prefix("xml");
    let xml_ns = xot.add_namespace("http://www.w3.org/XML/1998/namespace");
    let p0 = xot.first_child(root_id).unwrap();
    // add a p1 that doesn't have a prefix
    let p1_name = xot.add_name_ns("p", b_ns);
    xot.append_element(root_id, p1_name).unwrap();
    let p1 = xot.next_sibling(p0).unwrap();

    assert_eq!(
        xot.namespaces(p0).collect::<Vec<_>>(),
        [(a_prefix, a_ns), (xml_prefix, xml_ns)]
    );
    assert_eq!(
        xot.namespaces(root_id).collect::<Vec<_>>(),
        [(a_prefix, a_ns), (xml_prefix, xml_ns)]
    );

    assert_eq!(
        xot.namespaces(p1).collect::<Vec<_>>(),
        [(a_prefix, a_ns), (xml_prefix, xml_ns)]
    );
}

#[test]
fn test_namespaces2() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc xmlns="http://example.com/a"><p/></doc>"#)
        .unwrap();
    let root_id = xot.document_element(doc).unwrap();
    let empty_prefix = xot.empty_prefix();
    let a_ns = xot.add_namespace("http://example.com/a");
    let b_ns = xot.add_namespace("http://example.com/b");
    let xml_prefix = xot.add_prefix("xml");
    let xml_ns = xot.add_namespace("http://www.w3.org/XML/1998/namespace");
    let p0 = xot.first_child(root_id).unwrap();
    // add a p1 that doesn't have a prefix
    let p1_name = xot.add_name_ns("p", b_ns);
    xot.append_element(root_id, p1_name).unwrap();
    let p1 = xot.next_sibling(p0).unwrap();

    assert_eq!(
        xot.namespaces(p0).collect::<Vec<_>>(),
        [(empty_prefix, a_ns), (xml_prefix, xml_ns)]
    );
    assert_eq!(
        xot.namespaces(root_id).collect::<Vec<_>>(),
        [(empty_prefix, a_ns), (xml_prefix, xml_ns)]
    );

    assert_eq!(
        xot.namespaces(p1).collect::<Vec<_>>(),
        [(empty_prefix, a_ns), (xml_prefix, xml_ns)]
    );
}

#[test]
fn test_namespaces_overrides() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc xmlns:a="http://example.com/a"><a:p xmlns:a="http://example.com/b"/></doc>"#)
        .unwrap();
    let root_id = xot.document_element(doc).unwrap();
    let a_prefix = xot.add_prefix("a");
    let a_ns = xot.add_namespace("http://example.com/a");
    let b_ns = xot.add_namespace("http://example.com/b");
    let xml_prefix = xot.add_prefix("xml");
    let xml_ns = xot.add_namespace("http://www.w3.org/XML/1998/namespace");
    let p0 = xot.first_child(root_id).unwrap();
    // add a p1 that doesn't have a prefix
    let p1_name = xot.add_name_ns("p", b_ns);
    xot.append_element(root_id, p1_name).unwrap();
    let p1 = xot.next_sibling(p0).unwrap();

    assert_eq!(
        xot.namespaces(p0).collect::<Vec<_>>(),
        [(a_prefix, b_ns), (xml_prefix, xml_ns)]
    );
    assert_eq!(
        xot.namespaces(root_id).collect::<Vec<_>>(),
        [(a_prefix, a_ns), (xml_prefix, xml_ns)]
    );
    assert_eq!(
        xot.namespaces(p1).collect::<Vec<_>>(),
        [(a_prefix, a_ns), (xml_prefix, xml_ns)]
    );
}

#[test]
fn test_namespaces_overrides_xml_prefix() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc><p xmlns:xml="http://example.com/a"/></doc>"#)
        .unwrap();
    let root_id = xot.document_element(doc).unwrap();
    let a_ns = xot.add_namespace("http://example.com/a");
    let xml_prefix = xot.add_prefix("xml");
    let xml_ns = xot.add_namespace("http://www.w3.org/XML/1998/namespace");
    let p0 = xot.first_child(root_id).unwrap();

    assert_eq!(xot.namespaces(p0).collect::<Vec<_>>(), [(xml_prefix, a_ns)]);
    assert_eq!(
        xot.namespaces(root_id).collect::<Vec<_>>(),
        [(xml_prefix, xml_ns)]
    );
}
