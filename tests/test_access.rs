use xot::Xot;

#[test]
fn test_text_content_str() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a>text</a>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    assert_eq!(xot.text_content_str(doc_el), Some("text"));
}

#[test]
fn test_text_content_str_no_text() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a/>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    assert_eq!(xot.text_content_str(doc_el), Some(""));
}

#[test]
fn test_text_content_str_mixed_content() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a>text<b/></a>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    assert_eq!(xot.text_content_str(doc_el), None);
}

#[test]
fn test_compare() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<a>text</a>"#).unwrap();
    let doc2 = xot.parse(r#"<a>text</a>"#).unwrap();

    assert!(xot.compare(doc1, doc2));
}

#[test]
fn test_compare_different_text() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<a>text A</a>"#).unwrap();
    let doc2 = xot.parse(r#"<a>text B</a>"#).unwrap();

    assert!(!xot.compare(doc1, doc2));
}

#[test]
fn test_compare_different_structure() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<a></a>"#).unwrap();
    let doc2 = xot.parse(r#"<a><b/></a>"#).unwrap();

    assert!(!xot.compare(doc1, doc2));
}

#[test]
fn test_compare_different_namespace() {
    let mut xot = Xot::new();
    let doc1 = xot
        .parse(r#"<a xmlns="http://example.com/a"></a>"#)
        .unwrap();
    let doc2 = xot
        .parse(r#"<a xmlns="http://example.com/b"></a>"#)
        .unwrap();

    assert!(!xot.compare(doc1, doc2));
}

#[test]
fn test_compare_same_attributes() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<a x="X"></a>"#).unwrap();
    let doc2 = xot.parse(r#"<a x="X"></a>"#).unwrap();

    assert!(xot.compare(doc1, doc2));
}

#[test]
fn test_compare_different_attributes() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<a x="X"></a>"#).unwrap();
    let doc2 = xot.parse(r#"<a x="Y"></a>"#).unwrap();

    assert!(!xot.compare(doc1, doc2));
}

#[test]
fn test_compare_different_attributes_extra() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<a x="X"></a>"#).unwrap();
    let doc2 = xot.parse(r#"<a x="X" y="Y"></a>"#).unwrap();

    assert!(!xot.compare(doc1, doc2));
}

#[test]
fn test_compare_value_the_same_structure_different() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<article><body><sec id="1"><sec id="2"><title>T1</title><p>P1</p><p>P2</p></sec></sec><sec id="3"><title>T2</title><p>P3</p></sec></body></article>"#
    ).unwrap();
    let doc2 = xot.parse(r#"<article><body><sec id="1"><sec id="2"><title>T1</title><p>P1</p><p>P2</p></sec><sec id="3"><title>T2</title><p>P3</p></sec></sec></body></article>"#).unwrap();
    assert!(!xot.compare(doc1, doc2));
}
