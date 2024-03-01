use crate::serializer::Output;
use crate::xmlvalue::ValueType;
use crate::xotdata::{Node, Xot};
use crate::NameId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Space {
    Empty,
    Default,
    Preserve,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StackEntry {
    Unmixed(Space),
    Mixed,
}

pub(crate) struct Pretty<'a> {
    xot: &'a Xot,
    suppress: Vec<NameId>,
    stack: Vec<StackEntry>,
}

impl<'a> Pretty<'a> {
    pub(crate) fn new(xot: &'a Xot, suppress: Vec<NameId>) -> Pretty<'a> {
        Pretty {
            xot,
            suppress,
            stack: Vec::new(),
        }
    }

    fn unmixed(&mut self, space: Space) {
        self.stack.push(StackEntry::Unmixed(space));
    }

    fn mixed(&mut self) {
        self.stack.push(StackEntry::Mixed);
    }

    fn in_mixed(&self) -> bool {
        self.stack.iter().any(|e| *e == StackEntry::Mixed)
    }

    fn in_space_preserve(&self) -> bool {
        for entry in self.stack.iter().rev() {
            match entry {
                StackEntry::Unmixed(Space::Preserve) => return true,
                StackEntry::Unmixed(Space::Default) => return false,
                StackEntry::Unmixed(Space::Empty) => (),
                StackEntry::Mixed => return false,
            }
        }
        false
    }

    fn pop(&mut self) {
        self.stack.pop();
    }

    fn get_indentation(&self) -> usize {
        if self.in_mixed() {
            return 0;
        }
        let mut count = 0;
        let mut in_preserve = false;
        for entry in self.stack.iter() {
            match entry {
                StackEntry::Unmixed(Space::Default) => {
                    in_preserve = false;
                    count += 1
                }
                StackEntry::Unmixed(Space::Preserve) => in_preserve = true,
                StackEntry::Unmixed(Space::Empty) => {
                    if !in_preserve {
                        count += 1
                    }
                }
                StackEntry::Mixed => (),
            }
        }
        count
    }

    fn get_newline(&self) -> bool {
        !self.in_mixed() && !self.in_space_preserve()
    }

    fn has_text_child(&self, node: Node) -> bool {
        self.xot
            .children(node)
            .any(|child| self.xot.value_type(child) == ValueType::Text)
    }

    fn element_space(&self, node: Node) -> Space {
        let attributes = self.xot.attributes(node);
        let space = attributes
            .get(self.xot.xml_space_name())
            .map(|s| s.as_str());
        match space {
            Some("preserve") => Space::Preserve,
            Some("default") => Space::Default,
            _ => Space::Empty,
        }
    }

    pub(crate) fn prettify(&mut self, node: Node, output_token: &Output) -> (usize, bool) {
        use Output::*;
        match output_token {
            StartTagOpen(_) => (self.get_indentation(), false),
            Comment(_) | ProcessingInstruction(..) => (self.get_indentation(), self.get_newline()),
            StartTagClose => {
                let newline = if self.xot.first_child(node).is_some() {
                    if !self.has_text_child(node) {
                        let space = self.element_space(node);

                        self.unmixed(space);
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

    use crate::output;

    use crate::serialize::SerializeOptions;

    #[rstest]
    fn pretty(
        #[values(
            ("elements", r#"<doc><a><b/></a></doc>"#, vec![]),
            ("more elements", r#"<doc><a><b/></a><a><b/><b/></a></doc>"#, vec![]),
            ("text", r#"<doc><a>text</a><a>text 2</a></doc>"#, vec![]),
            ("mixed", r#"<doc><p>Hello <em>world</em>!</p></doc>"#, vec![]),
            ("mixed, nested", r#"<doc><p>Hello <em><strong>world</strong></em>!</p></doc>"#, vec![]),
            ("mixed, multi", r#"<doc><p>Hello <em>world</em>!</p><p>Greetings, <strong>universe</strong>!</p></doc>"#, vec![]),
            // the embedded content could in principle be pretty printed, we don't. This is similar to xmllint --format behavior
            ("mixed, embedded", r#"<doc><p>Hello <nested><stuff>a</stuff><stuff>b</stuff></nested></p></doc>"#, vec![]),
            ("comment", r#"<doc><a><!--hello--><!--world--></a></doc>"#, vec![]),
            ("mixed, comment", r#"<doc><p>Hello <!--world-->!</p></doc>"#, vec![]),
            ("multi pi", r#"<doc><a><?pi?><?pi?></a></doc>"#, vec![]),
            ("preserve", r#"<doc xml:space="preserve"><p>Hello</p></doc>"#, vec![]),
            ("preserve_nested", r#"<doc xml:space="preserve">  <p><foo>  </foo></p></doc>"#, vec![]),
            ("preserve_back_to_default", r#"<doc xml:space="preserve"><p xml:space="default"><foo><bar/></foo></p></doc>"#, vec![]),
            ("not suppressed", r#"<doc><a><b/></a></doc>"#, vec![]),
            // ("suppressed", r#"<doc><a><b/></a></doc>"#, vec!["a"]),
        )]
        value: (&str, &str, Vec<&str>),
    ) {
        let (name, xml, suppress) = value;
        let mut xot = Xot::new();
        let suppress = suppress.iter().map(|s| xot.add_name(s)).collect();

        let document = xot.parse(xml).unwrap();
        let output_xml = xot
            .serialize_xml(
                output::xml::Parameters {
                    indentation: Some(output::xml::Indentation { suppress }),
                    ..Default::default()
                },
                document,
            )
            .unwrap();
        assert_snapshot!(name, output_xml, xml);
    }
}
