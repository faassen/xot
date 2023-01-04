// A static representation of a tree of nodes
// Xot is a dynamic system, but for the purposes of proptests it's
// useful to have a static representation of a tree of nodes.
use crate::error::Error;
use crate::xmlvalue::{Comment, Element, ProcessingInstruction, Text};
use crate::xotdata::{Node, Xot};

enum Fixed {
    Text(Text),
    Comment(Comment),
    ProcessingInstruction(ProcessingInstruction),
    Root(Vec<RootContent>, Box<Fixed>, Vec<RootContent>),
    Element(Element, Vec<Fixed>),
}

enum RootContent {
    Comment(Comment),
    ProcessingInstruction(ProcessingInstruction),
}

impl Fixed {
    fn xotify(&self, xot: &mut Xot) -> Result<Node, Error> {
        Ok(match self {
            Fixed::Text(text) => xot.new_text(text.get()),
            Fixed::Comment(comment) => xot.new_comment(comment.get()),
            Fixed::ProcessingInstruction(pi) => {
                xot.new_processing_instruction(pi.target(), pi.data())
            }
            Fixed::Root(before, child, after) => {
                let child = child.xotify(xot)?;
                let root = xot.new_root(child)?;
                for content in before {
                    let node = create_root_content_node(xot, content);
                    xot.insert_before(root, node)?;
                }
                for content in after {
                    let node = create_root_content_node(xot, content);
                    xot.append(root, node)?;
                }
                root
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

fn create_root_content_node(xot: &mut Xot, content: &RootContent) -> Node {
    match content {
        RootContent::Comment(comment) => xot.new_comment(comment.get()),
        RootContent::ProcessingInstruction(pi) => {
            xot.new_processing_instruction(pi.target(), pi.data())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xotify() {
        let mut xot = Xot::new();
        let name_id = xot.add_name("foo");
        let root = Fixed::Root(
            vec![],
            Box::new(Fixed::Element(
                Element::new(name_id),
                vec![Fixed::Text(Text::new("Example".to_string()))],
            )),
            vec![],
        );
        let root = root.xotify(&mut xot).unwrap();
        assert_eq!(xot.serialize_to_string(root), "<foo>Example</foo>");
    }
}
