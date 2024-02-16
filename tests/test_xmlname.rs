use xot::{
    xmlname::{NameIdInfo, NameStrInfo, XmlNameOwned},
    Xot,
};

#[test]
fn test_owned() {
    let name = XmlNameOwned::new(
        "local".to_string(),
        "http://example.com".to_string(),
        "prefix".to_string(),
    );

    assert_eq!(name.local_name(), "local");
    assert_eq!(name.namespace(), "http://example.com");
    assert_eq!(name.prefix().unwrap(), "prefix");
    assert_eq!(name.full_name().unwrap(), "prefix:local")
}

#[test]
fn test_ref() {
    let name = XmlNameOwned::new(
        "local".to_string(),
        "http://example.com".to_string(),
        "prefix".to_string(),
    );

    let mut xot = Xot::new();
    let namespace_id = xot.add_namespace("http://example.com");
    let name_id = xot.add_name_ns("local", namespace_id);
    let prefix_id = xot.add_prefix("prefix");

    let name_ref = name.to_ref(&mut xot);
    assert_eq!(name_ref.local_name(), "local");
    assert_eq!(name_ref.namespace(), "http://example.com");
    assert_eq!(name_ref.prefix().unwrap(), "prefix");
    assert_eq!(name_ref.full_name().unwrap(), "prefix:local");
    assert_eq!(name_ref.to_owned().unwrap(), name);
    assert_eq!(name_ref.name_id(), name_id);
    assert_eq!(name_ref.namespace_id(), namespace_id);
    assert_eq!(name_ref.prefix_id().unwrap(), prefix_id);
}

#[test]
fn test_state() {
    let name = XmlNameOwned::new(
        "local".to_string(),
        "http://example.com".to_string(),
        "prefix".to_string(),
    );

    let mut xot = Xot::new();
    let namespace_id = xot.add_namespace("http://example.com");
    let name_id = xot.add_name_ns("local", namespace_id);
    let prefix_id = xot.add_prefix("prefix");

    let name_state = name.to_state(&mut xot).unwrap();
    assert_eq!(name_state.name_id(), name_id);
    assert_eq!(name_state.namespace_id(), namespace_id);
    assert_eq!(name_state.prefix_id().unwrap(), prefix_id);
}
