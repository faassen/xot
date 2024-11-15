use xot::Xot;

#[test]
fn test_parse_invalid_close_tag() {
    let xml = r#"<doc></a></doc>"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert!(matches!(err, xot::ParseError::InvalidCloseTag { .. }));
    assert_eq!(err.span(), (7..8).into());
}

#[test]
fn test_parse_invalid_close_tag_prefix() {
    let xml = r#"<doc xmlns:a="http://example.com"></a:doc></doc>"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert!(matches!(err, xot::ParseError::InvalidCloseTag { .. }));
    assert_eq!(err.span(), (36..41).into());
}

#[test]
fn test_unknown_prefix() {
    let xml = r#"<doc><a:p/></doc>"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert!(matches!(err, xot::ParseError::UnknownPrefix { .. }));
    assert_eq!(err.span(), (6..7).into());
}

#[test]
fn test_unsupported_version() {
    let xml = r#"<?xml version="1.1"?><doc/></doc>"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert!(matches!(err, xot::ParseError::UnsupportedVersion { .. }));
    assert_eq!(err.span(), (15..18).into());
}

#[test]
fn test_unsupported_not_standalone() {
    let xml = r#"<?xml version="1.0" standalone="no"?><doc></doc>"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert!(matches!(
        err,
        xot::ParseError::UnsupportedNotStandalone { .. }
    ));
    assert_eq!(err.span(), (0..37).into());
}

#[test]
fn test_xmlparser_error() {
    let xml = r#"<doc><"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert!(matches!(err, xot::ParseError::XmlParser { .. }));
    assert_eq!(err.span(), (5..5).into());
}

#[test]
fn test_duplicate_attribute() {
    let xml = r#"<doc a="1" a="2"/>"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert!(matches!(err, xot::ParseError::DuplicateAttribute { .. }));
    assert_eq!(err.span(), (11..12).into());
}

#[test]
fn test_duplicate_attribute_with_prefix() {
    let xml = r#"<doc xmlns:a="http://example.com" a:b="1" a:b="2"/>"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert!(matches!(err, xot::ParseError::DuplicateAttribute { .. }));
    assert_eq!(err.span(), (42..45).into());
}

#[test]
fn test_invalid_dtd() {
    let xml = r#"<!DOCTYPE note><note></note>"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert!(matches!(err, xot::ParseError::DtdUnsupported { .. }));
    assert_eq!(err.span(), (0..15).into());
}

#[test]
fn test_dtd_unsupported() {
    let xml = r#"<!DOCTYPE note SYSTEM "Note.dtd"><note></note>"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert!(matches!(err, xot::ParseError::DtdUnsupported { .. }));
    assert_eq!(err.span(), (0..33).into());
}

#[test]
fn test_dtd_unsupported2() {
    let xml = r#"<?xml version="1.0"?><!DOCTYPE note SYSTEM "Note.dtd"><note></note>"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert!(matches!(err, xot::ParseError::DtdUnsupported { .. }));
    assert_eq!(err.span(), (21..54).into());
}

#[test]
fn test_unclosed_tag() {
    let xml = r#"<doc>"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert!(matches!(err, xot::ParseError::UnclosedTag(_)));
    assert_eq!(err.span(), (1..4).into());
}

#[test]
fn test_unclosed_tag_middle() {
    let xml = r#"<doc><a></doc>"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert!(matches!(err, xot::ParseError::InvalidCloseTag(_, _, _)));
    assert_eq!(err.span(), (10..13).into());
}

#[test]
fn test_unclosed_tag_with_prefix() {
    let xml = r#"<a:doc xmlns:a="http://example.com">"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert!(matches!(err, xot::ParseError::UnclosedTag(_)));
    assert_eq!(err.span(), (1..6).into());
}

#[test]
fn test_only_text_top_level() {
    let xml = r#"abc"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert!(matches!(err, xot::ParseError::XmlParser { .. }));
    assert_eq!(err.span(), (0..0).into());
}

#[test]
fn test_only_comment_top_level() {
    let xml = r#"<!-- foo -->"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert!(matches!(err, xot::ParseError::NoElementAtTopLevel { .. }));
    assert_eq!(err.span(), (12..12).into());
}
