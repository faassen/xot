use xot::{Document, Error, XmlData};

#[test]
fn test_unclosed_tag() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<a><b></a>"#, &mut data);
    assert!(matches!(doc, Err(Error::InvalidCloseTag(_, _))));
}

#[test]
fn test_unclosed_tag_at_end() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<a>"#, &mut data);
    assert!(matches!(doc, Err(Error::UnclosedTag)));
}
