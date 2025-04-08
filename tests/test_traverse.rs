use xot::{NodeEdge, Xot};

#[test]
fn test_traverse() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a><b foo="FOO">text</b></a>"#).unwrap();
    let a = xot.document_element(doc).unwrap();

    let b = xot.first_child(a).unwrap();
    let text = xot.first_child(b).unwrap();

    let mut result = Vec::new();
    for node in xot.traverse(doc) {
        result.push(node);
    }

    assert_eq!(
        result,
        vec![
            NodeEdge::Start(doc),
            NodeEdge::Start(a),
            NodeEdge::Start(b),
            NodeEdge::Start(text),
            NodeEdge::End(text),
            NodeEdge::End(b),
            NodeEdge::End(a),
            NodeEdge::End(doc),
        ]
    );
}

#[test]
fn test_reverse_traverse() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a><b foo="FOO">text</b></a>"#).unwrap();
    let a = xot.document_element(doc).unwrap();

    let b = xot.first_child(a).unwrap();
    let text = xot.first_child(b).unwrap();

    let mut result = Vec::new();
    for node in xot.reverse_traverse(doc) {
        result.push(node);
    }

    assert_eq!(
        result,
        vec![
            NodeEdge::End(doc),
            NodeEdge::End(a),
            NodeEdge::End(b),
            NodeEdge::End(text),
            NodeEdge::Start(text),
            NodeEdge::Start(b),
            NodeEdge::Start(a),
            NodeEdge::Start(doc),
        ]
    );
}

#[test]
fn test_reverse_preorder() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a><b foo="FOO">text</b><c/></a>"#).unwrap();
    let a = xot.document_element(doc).unwrap();

    let b = xot.first_child(a).unwrap();
    let text = xot.first_child(b).unwrap();
    let c = xot.next_sibling(b).unwrap();

    let result = xot.reverse_preorder(c).collect::<Vec<_>>();

    assert_eq!(result, vec![c, text, b, a, doc]);
}

#[test]
fn test_all_reverse_preorder() {
    let mut xot = Xot::new();
    let foo = xot.add_name("foo");
    let doc = xot.parse(r#"<a><b foo="FOO">text</b><c/></a>"#).unwrap();
    let a = xot.document_element(doc).unwrap();

    let b = xot.first_child(a).unwrap();
    let foo = xot.attributes(b).get_node(foo).unwrap();
    let text = xot.first_child(b).unwrap();
    let c = xot.next_sibling(b).unwrap();

    let result = xot.all_reverse_preorder(c).collect::<Vec<_>>();

    assert_eq!(result, vec![c, text, foo, b, a, doc]);
}

#[test]
fn test_all_reverse_preorder2() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<a><b><c><d/><e/></c><f/></b><g/></a>"#)
        .unwrap();
    let a = xot.document_element(doc).unwrap();
    let b = xot.first_child(a).unwrap();
    let c = xot.first_child(b).unwrap();
    let d = xot.first_child(c).unwrap();
    let e = xot.next_sibling(d).unwrap();
    let f = xot.next_sibling(c).unwrap();
    let g = xot.next_sibling(b).unwrap();

    let result = xot.all_reverse_preorder(g).collect::<Vec<_>>();
    assert_eq!(result, vec![g, f, e, d, c, b, a, doc]);
}
