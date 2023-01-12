use crate::serializer::OutputToken;
use crate::xmlvalue::ValueType;
use crate::xotdata::{Node, Xot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StackEntry {
    Unmixed,
    Mixed,
}

pub(crate) struct Pretty<'a> {
    xot: &'a Xot<'a>,
    stack: Vec<StackEntry>,
}

impl<'a> Pretty<'a> {
    pub(crate) fn new(xot: &'a Xot<'a>) -> Pretty<'a> {
        Pretty {
            xot,
            stack: Vec::new(),
        }
    }

    fn unmixed(&mut self) {
        self.stack.push(StackEntry::Unmixed);
    }

    fn mixed(&mut self) {
        self.stack.push(StackEntry::Mixed);
    }

    fn in_mixed(&self) -> bool {
        self.stack.iter().any(|e| *e == StackEntry::Mixed)
    }

    fn pop(&mut self) {
        self.stack.pop();
    }

    fn get_indentation(&self) -> usize {
        if self.in_mixed() {
            return 0;
        }
        self.stack
            .iter()
            .filter(|e| **e == StackEntry::Unmixed)
            .count()
    }

    fn get_newline(&self) -> bool {
        !self.in_mixed()
    }

    fn has_text_child(&self, node: Node) -> bool {
        self.xot
            .children(node)
            .any(|child| self.xot.value_type(child) == ValueType::Text)
    }

    pub(crate) fn prettify(&mut self, node: Node, output_token: &OutputToken) -> (usize, bool) {
        use OutputToken::*;
        match output_token {
            StartTagOpen(_) => (self.get_indentation(), false),
            Comment(_) | ProcessingInstruction(..) => (self.get_indentation(), self.get_newline()),
            StartTagClose(..) => {
                let newline = if self.xot.first_child(node).is_some() {
                    if !self.has_text_child(node) {
                        self.unmixed();
                        self.get_newline()
                    } else {
                        self.mixed();
                        false
                    }
                } else {
                    false
                };
                (0, newline)
            }
            EndTag(_) => {
                let indentation = if self.xot.first_child(node).is_some() {
                    let was_in_mixed = self.in_mixed();
                    self.pop();
                    if !was_in_mixed {
                        self.get_indentation()
                    } else {
                        0
                    }
                } else {
                    0
                };
                (indentation, self.get_newline())
            }
            _ => (0, false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use rstest::rstest;

    use crate::serialize::SerializeOptions;

    #[rstest]
    fn pretty(
        #[values(
            r#"<doc><a><b/></a></doc>"#,
            r#"<doc><a><b/></a><a><b/><b/></a></doc>"#,
            r#"<doc><a>text</a><a>text 2</a></doc>"#,
            r#"<doc><p>Hello <em>world</em>!</p></doc>"#,
            r#"<doc><p>Hello <em><strong>world</strong></em>!</p></doc>"#,
            r#"<doc><p>Hello <em>world</em>!</p><p>Greetings, <strong>universe</strong>!</p></doc>"#,
            r#"<doc><a><!--hello--><!--world--></a></doc>"#,
            r#"<doc><p>Hello <!--world-->!</p></doc>"#,
            r#"<doc><a><?pi?><?pi?></a></doc>"#
        )]
        xml: &str,
    ) {
        let mut xot = Xot::new();
        let root = xot.parse(xml).unwrap();
        let output_xml = xot
            .serialize_options(SerializeOptions { pretty: true })
            .to_string(root)
            .unwrap();
        assert_snapshot!(xml, output_xml);
    }
}
