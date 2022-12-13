use xot::{Error, Xot};

fn main() -> Result<(), Error> {
    let mut xot = Xot::new();

    let doc = xot.parse("<p>Example</p>")?;
    let doc_el = xot.document_element(doc)?;
    let txt_node = xot.first_child(doc_el).unwrap();
    let txt_value = xot.text(txt_node).unwrap();
    assert_eq!(txt_value.get(), "Example");
    Ok(())
}
