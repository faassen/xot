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
