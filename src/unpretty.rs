use crate::xotdata::{Node, Xot};

pub(crate) fn remove_insignificant_whitespace(xot: &mut Xot, node: Node) {
    let mut to_remove = Vec::new();
    for descendant in xot.descendants(node) {
        if is_insignificant_whitespace(xot, descendant) {
            to_remove.push(descendant);
        }
    }
    for node in to_remove {
        xot.remove(node).unwrap();
    }
}

fn is_whitespace(text: &str) -> bool {
    text.chars().all(|c| c.is_whitespace())
}

fn is_significant_text_node(xot: &Xot, node: Node) -> bool {
    if let Some(text) = xot.text_str(node) {
        !is_whitespace(text)
    } else {
        false
    }
}

fn in_preserve_space(xot: &Xot, node: Node) -> bool {
    let space = xot.xml_space_name();
    for ancestor in xot.ancestors(node) {
        let attributes = xot.attributes(ancestor);
        if let Some(value) = attributes.get(&space) {
            return value == "preserve";
        }
    }
    false
}

fn is_insignificant_whitespace(xot: &Xot, node: Node) -> bool {
    if let Some(text) = xot.text_str(node) {
        if in_preserve_space(xot, node) {
            return false;
        }
        if !is_whitespace(text) {
            return false;
        }
        let previous_sibling = xot.previous_sibling(node);
        if let Some(previous_sibling) = previous_sibling {
            if xot
                .preceding_siblings(previous_sibling)
                .any(|n| is_significant_text_node(xot, n))
            {
                return false;
            }
        }
        let next_sibling = xot.next_sibling(node);
        if let Some(next_sibling) = next_sibling {
            if xot
                .following_siblings(next_sibling)
                .any(|n| is_significant_text_node(xot, n))
            {
                return false;
            }
        }
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unpretty() {
        let mut xot = Xot::new();
        let root = xot
            .parse("<doc>  <p>hello <i>world</i>  </p>  </doc>")
            .unwrap();
        remove_insignificant_whitespace(&mut xot, root);
        assert_eq!(
            xot.to_string(root).unwrap(),
            "<doc><p>hello <i>world</i>  </p></doc>"
        );
    }

    #[test]
    fn test_unpretty_xml_space_preserve() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc xml:space="preserve">   </doc>"#).unwrap();
        remove_insignificant_whitespace(&mut xot, root);
        assert_eq!(
            xot.to_string(root).unwrap(),
            r#"<doc xml:space="preserve">   </doc>"#
        );
    }

    #[test]
    fn test_unpretty_xml_space_preserve_nested() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<doc xml:space="preserve"><p>   </p></doc>"#)
            .unwrap();
        remove_insignificant_whitespace(&mut xot, root);
        assert_eq!(
            xot.to_string(root).unwrap(),
            r#"<doc xml:space="preserve"><p>   </p></doc>"#
        );
    }

    #[test]
    fn test_unpretty_xml_space_preserve_reset() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<doc xml:space="preserve">  <p xml:space="default">   </p></doc>"#)
            .unwrap();
        remove_insignificant_whitespace(&mut xot, root);
        assert_eq!(
            xot.to_string(root).unwrap(),
            r#"<doc xml:space="preserve">  <p xml:space="default"/></doc>"#
        );
    }
}
