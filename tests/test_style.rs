use xot::{Error, Node, OutputToken, Pretty, SerializationData, Serializer, Xot};

#[test]
fn test_style() -> Result<(), Error> {
    // we have XML with in it particular `style` wrappers around text nodes
    // We want to render this XML to HTML (so the XML is escaped) with the
    // style wrappers removed and replaced by a <span>. We want the resulting
    // rendered XML to be pretty printed.
    //
    // We do this by using the advanced serialization features
    let mut xot = Xot::new();

    let root = xot.parse("<a><b><style>foo</style></b></a>")?;
    // expected:
    // <a>
    //  <b><span>foo</span></b>
    // </a>
    // because foo is text content, and thus span shouldn't be indented or affected
    // we currently determine text content by using the xot structure, but
    // we should instead peek ahead in the stream to determine a text node
    // so the stream should be ahead by all children of the current node
    // But the problem with that is that it expands the full tree, because
    // descendants are children too. We could peek until the first text node
    // is encountered each time (that's not a descendant). In the worst case
    // this is a huge deal of nodes, though.
    // This way Pretty doesn't need access to xot anymore.
    let name_style = xot.add_name("style");

    let mut serializer = xot.serializer(root);
    let output_tokens = xot.output_tokens(root);
    let mut pretty = xot.pretty();

    // we want to go through the tokens and display them in an indented way
    // but we can't do it correctly as prettier uses tree structure to determine
    // indentation. We'd like prettier to only use the stream of actual output
    // tokens to determine indentation. We could produce the stream of final output
    // tokens as well as marking which parts of them we want to handle with special
    // style. We can uniquely identify using a combination of node and attribute to
    // identify attributes, just node to identify other nodes.
    // We create a stream of output tokens with indentation information, and then
    // use the markings during a final rendering pass.
    // The prettier indenter would work in terms of an iterator of output tokens
    // only: it can do this by maintaining a stack and running ahead to determine
    // whether an element is mixed and thus shouldn't have indentation.

    #[derive(Debug, PartialEq, Eq)]
    enum Style {
        Start,
        End,
        Text(String),
    }

    let mut output = Vec::new();

    use OutputToken::*;

    fn render_normal(
        indentation: usize,
        newline: bool,
        rendered: SerializationData,
        output: &mut Vec<Style>,
    ) {
        if indentation > 0 {
            output.push(Style::Text(" ".repeat(indentation * 2)));
        }
        if rendered.space {
            output.push(Style::Text(" ".to_string()));
        }
        output.push(Style::Text(rendered.text));
        if newline {
            output.push(Style::Text("\n".to_string()));
        }
    }

    for (node, output_token) in output_tokens {
        match output_token {
            StartTagOpen(element) => {
                if element.name() == name_style {
                    output.push(Style::Start);
                } else {
                    let (indentation, newline) = pretty.prettify(node, &output_token);
                    let rendered = serializer.render(node, output_token)?;
                    render_normal(indentation, newline, rendered, &mut output);
                }
            }
            StartTagClose(element) => {
                if element.name() == name_style {
                    // do nothing
                } else {
                    let (indentation, newline) = pretty.prettify(node, &output_token);
                    let rendered = serializer.render(node, output_token)?;
                    render_normal(indentation, newline, rendered, &mut output);
                }
            }
            EndTag(element) => {
                if element.name() == name_style {
                    output.push(Style::End);
                } else {
                    let (indentation, newline) = pretty.prettify(node, &output_token);
                    let rendered = serializer.render(node, output_token)?;
                    render_normal(indentation, newline, rendered, &mut output);
                }
            }
            _ => {
                let (indentation, newline) = pretty.prettify(node, &output_token);
                let rendered = serializer.render(node, output_token)?;
                render_normal(indentation, newline, rendered, &mut output);
            }
        }
    }
    let output = output
        .iter()
        .map(|style| match style {
            Style::Start => "<span>".to_string(),
            Style::End => "</span>".to_string(),
            Style::Text(text) => text.to_string(),
        })
        .collect::<Vec<_>>()
        .join("");

    assert_eq!(output, "");
    Ok(())
}
