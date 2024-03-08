use markdown_it::{
    parser::inline::{InlineRule, InlineState},
    MarkdownIt,
    Node,
    NodeValue,
    Renderer,
};

pub fn markdown_parser() -> MarkdownIt {
    let mut parser = MarkdownIt::new();
    markdown_it::plugins::cmark::add(&mut parser);
    markdown_it::plugins::extra::add(&mut parser);
    parser.inline.add_rule::<ArticleLinkScanner>();
    parser
}

#[derive(Debug)]
pub struct ArticleLink {
    title: String,
    domain: String,
}

// This defines how your custom node should be rendered.
impl NodeValue for ArticleLink {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let mut attrs = node.attrs.clone();

        let link = format!("/article/{}@{}", self.title, self.domain);
        attrs.push(("href", link));

        fmt.open("a", &attrs);
        fmt.text(&self.title);
        fmt.close("a");
    }
}

struct ArticleLinkScanner;

impl InlineRule for ArticleLinkScanner {
    const MARKER: char = '[';

    /// Find `[[Title@example.com]], return the position and split title/domain.
    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let input = &state.src[state.pos..state.pos_max];
        if !input.starts_with("[[") {
            return None;
        }
        const SEPARATOR_LENGTH: usize = 2;

        input.find("]]").and_then(|length| {
            let start = state.pos + SEPARATOR_LENGTH;
            let i = start + length - SEPARATOR_LENGTH;
            let content = &state.src[start..i];
            content.split_once('@').map(|(title, domain)| {
                let node = Node::new(ArticleLink {
                    title: title.to_string(),
                    domain: domain.to_string(),
                });
                (node, length + SEPARATOR_LENGTH)
            })
        })
    }
}

#[test]
fn test_markdown_article_link() {
    let parser = markdown_parser();
    let rendered = parser
        .parse("some text [[Title@example.com]] and more")
        .render();
    assert_eq!(
        "<p>some text <a href=\"/article/Title@example.com\">Title</a> and more</p>\n",
        rendered
    );
}
