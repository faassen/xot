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

#[test]
fn test_duplicate_attributes() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<a x="x" x="y"/>"#);
    if let Err(Error::DuplicateAttribute(s)) = doc {
        assert_eq!(s, "x");
    } else {
        unreachable!();
    }
}

#[test]
fn test_duplicate_attributes_ns() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<a xmlns:foo="http://example.com" foo:x="x" foo:x="y"/>"#);
    if let Err(Error::DuplicateAttribute(s)) = doc {
        assert_eq!(s, "foo:x");
    } else {
        unreachable!();
    }
}

#[test]
fn test_parse_xml_declaration() {
    let mut data = XmlData::new();
    let doc = data.parse(r#"<?xml version="1.0" encoding="UTF-8"?><a/>"#);
    assert!(doc.is_ok());
}
