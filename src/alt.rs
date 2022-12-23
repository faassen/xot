// a static tree generation system
// this is useful for generating trees for proptest purposes
use crate::error::Error;
use crate::name::NameId;
use crate::xmlvalue::{Comment, Element, ProcessingInstruction, Text};
use crate::xotdata::{Node, Xot};

// The plan:
// Proptest trees of nodes
// Turn into Xot
// Serialize to string
// Parse again with Xot
// See whether we get the same

enum Fixed {
    Text(Text),
    Comment(Comment),
    ProcessingInstruction(ProcessingInstruction),
    Root(Box<Fixed>),
    Element(Element, Vec<Fixed>),
}

impl Fixed {
    fn xotify(&self, xot: &mut Xot) -> Result<Node, Error> {
        Ok(match self {
            Fixed::Text(text) => xot.new_text(text.get()),
            Fixed::Comment(comment) => xot.new_comment(comment.get()),
            Fixed::ProcessingInstruction(pi) => {
                xot.new_processing_instruction(pi.target(), pi.data())
            }
            Fixed::Root(child) => {
                let child = child.xotify(xot)?;
                xot.new_root(child)?
            }
            Fixed::Element(element, children) => {
                let children = children
                    .iter()
                    .map(|child| child.xotify(xot))
                    .collect::<Result<Vec<_>, _>>()?;
                let element = xot.new_element(element.name());
                for child in children {
                    xot.append(element, child)?;
                }
                element
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xotify() {
        let mut xot = Xot::new();
        let name_id = xot.add_name("foo");
        let root = Fixed::Root(Box::new(Fixed::Element(
            Element::new(name_id),
            vec![Fixed::Text(Text::new("Example".to_string()))],
        )));
        let root = root.xotify(&mut xot).unwrap();
        assert_eq!(xot.serialize_to_string(root), "<foo>Example</foo>");
    }
}
