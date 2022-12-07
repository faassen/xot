use xot::{Error, XmlData};

#[test]
fn test_unclosed_tag() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<a><b></a>"#);
    assert!(matches!(doc, Err(Error::InvalidCloseTag(_, _))));
}

#[test]
fn test_unclosed_tag_at_end() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<a>"#);
    assert!(matches!(doc, Err(Error::UnclosedTag)));
}
