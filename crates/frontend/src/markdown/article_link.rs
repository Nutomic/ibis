use markdown_it::{
    Node,
    NodeValue,
    Renderer,
    parser::inline::{InlineRule, InlineState},
};

#[derive(Debug)]
struct ArticleLink {
    label: String,
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
        fmt.text(&self.label);
        fmt.close("a");
    }
}

pub struct ArticleLinkScanner;

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
                // Handle custom link label if provided, otherwise use title as label
                let (domain, label) = domain.split_once('|').unwrap_or((domain, title));
                let node = Node::new(ArticleLink {
                    label: label.to_string(),
                    title: title.to_string(),
                    domain: domain.to_string(),
                });
                (node, length + SEPARATOR_LENGTH)
            })
        })
    }
}

#[cfg(test)]
mod test {
    use crate::markdown::render_article_markdown;

    #[test]
    fn test_markdown_article_link() {
        let plain = render_article_markdown("[[Title@example.com]]");
        assert_eq!(
            "<p><a href=\"/article/Title@example.com\">Title</a></p>\n",
            plain
        );

        let with_label = render_article_markdown("[[Title@example.com|Example Article]]");
        assert_eq!(
            "<p><a href=\"/article/Title@example.com\">Example Article</a></p>\n",
            with_label
        );
    }
}
