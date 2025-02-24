use xot::Xot;

#[test]
fn test_readonly() {
    let mut xot = Xot::new();

    let r = xot.parse_readonly("<p>Example</p>").unwrap();
    let doc_el = xot.first_child(r).unwrap();
}
