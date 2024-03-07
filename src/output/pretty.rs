use crate::output::Output;
use crate::xmlvalue::ValueType;
use crate::xotdata::{Node, Xot};
use crate::{NameId, Value};

/// Pretty output token
///
/// Like [`OutputToken`](`crate::output::OutputToken`) but with extra information for
/// pretty printing.
pub struct PrettyOutputToken {
    /// indentation level.
    pub indentation: usize,
    /// Whether the token is prefixed by a space character.
    pub space: bool,
    /// The token
    ///
    /// This is a fragment of XML like `"<p"`, `a="A"` or `"</p>"`.
    pub text: String,
    /// Whether the token is suffixed by a newline character.
    pub newline: bool,
}

// we need to track where we are in xml:space, so that we can know when to
// insert newlines and indentation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Space {
    Empty,
    Default,
    Preserve,
}

// The stack keeps track of where we are, and the xml space state. We are
// either in a mixed element (with text and subcontent) (in which case we don't
// do any indentation anymore, including for its descendants), or in an element
// without text, in which case we can potentially indent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StackEntry {
    Unmixed(Space),
    Mixed,
}

pub(crate) struct Pretty<'a, IsSuppressed, IsInline>
where
    IsSuppressed: Fn(NameId) -> bool,
    IsInline: Fn(NameId) -> bool,
{
    xot: &'a Xot,
    is_suppressed: IsSuppressed,
    is_inline: IsInline,
    // a list of element names where we don't do indentation for the immediate content
    // suppress: &'a [NameId],
    stack: Vec<StackEntry>,
}

impl<'a, IsSuppressed, IsInline> Pretty<'a, IsSuppressed, IsInline>
where
    IsSuppressed: Fn(NameId) -> bool,
    IsInline: Fn(NameId) -> bool,
{
    pub(crate) fn new(xot: &'a Xot, is_suppressed: IsSuppressed, is_inline: IsInline) -> Self {
        Pretty {
            xot,
            is_suppressed,
            is_inline,
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

    fn has_inline_child(&self, node: Node) -> bool {
        // a node has an inline child if it has a text node, or an element
        // defined as inline
        self.xot
            .children(node)
            .any(|child| match self.xot.value(child) {
                Value::Text(_) => true,
                Value::Element(element) => (self.is_inline)(element.name()),
                _ => false,
            })
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
                    if !self.has_inline_child(node) {
                        let suppress = if let Some(element) = self.xot.element(node) {
                            (self.is_suppressed)(element.name())
                        } else {
                            false
                        };
                        // treat suppress as mixed content, as we don't want to indent
                        // anywhere inside
                        if suppress {
                            self.mixed();
                        } else {
                            let space = self.element_space(node);
                            self.unmixed(space);
                        }
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
                    let no_indentation = self.in_mixed();
                    self.pop();
                    if !no_indentation {
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
            ("suppressed", r#"<doc><a><b/></a></doc>"#, vec!["a"]),
            ("suppressed nested", r#"<doc><a><b><c/></b></a></doc>"#, vec!["a"]),
        )]
        value: (&str, &str, Vec<&str>),
    ) {
        let (name, xml, suppress) = value;
        let mut xot = Xot::new();
        let suppress = suppress.iter().map(|s| xot.add_name(s)).collect();

        let document = xot.parse(xml).unwrap();
        let output_xml = xot
            .serialize_xml_string(
                output::xml::Parameters {
                    indentation: Some(output::Indentation { suppress }),
                    ..Default::default()
                },
                document,
            )
            .unwrap();
        assert_snapshot!(name, output_xml, xml);
    }
}
