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
    assert_eq!(xot.text_content_str(doc_el), None);
}

#[test]
fn test_text_content_str_mixed_content() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a>text<b/></a>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    assert_eq!(xot.text_content_str(doc_el), None);
}
