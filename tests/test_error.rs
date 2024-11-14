use xot::Xot;

#[test]
fn test_parse_invalid_close_tag() {
    let xml = r#"<doc></a></doc>"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert_eq!(err.parse_error().unwrap().span(), (7..8).into());
}

#[test]
fn test_parse_invalid_close_tag_prefix() {
    let xml = r#"<doc xmlns:a="http://example.com"></a:doc></doc>"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert_eq!(err.parse_error().unwrap().span(), (36..41).into());
}

#[test]
fn test_unknown_prefix() {
    let xml = r#"<doc><a:p/></doc>"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert_eq!(err.parse_error().unwrap().span(), (6..7).into());
}

#[test]
fn test_unsupported_version() {
    let xml = r#"<?xml version="1.1"?><doc/></doc>"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    assert_eq!(err.parse_error().unwrap().span(), (15..18).into());
}

#[test]
fn test_unsupported_not_standalone() {
    let xml = r#"<?xml standalone="no"?><doc/></doc>"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    // this is a weird span, but it's what we get for the span for declaration
    // since declarations are only at the start, we'll live with it
    assert_eq!(err.parse_error().unwrap().span(), (0..0).into());
}

#[test]
fn test_parser_error() {
    let xml = r#"<doc><"#;
    let mut xot = Xot::new();
    let err = xot.parse(xml).unwrap_err();
    dbg!(&err);
    assert_eq!(err.parse_error().unwrap().span(), (5..5).into());
}
