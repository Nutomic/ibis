use markdown_it::{
    parser::inline::{InlineRule, InlineState},
    MarkdownIt,
    Node,
    NodeValue,
    Renderer,
};

use katex;

pub fn markdown_parser() -> MarkdownIt {
    let mut parser = MarkdownIt::new();
    markdown_it::plugins::cmark::add(&mut parser);
    markdown_it::plugins::extra::add(&mut parser);
    parser.inline.add_rule::<MathEquationScanner>();
    parser.inline.add_rule::<ArticleLinkScanner>();
    //parser.block.add_rule::<MathEquationScanner>();
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

#[derive(Debug)]
pub struct MathEquation {
    equation: String,
    display_mode: bool,
}

impl NodeValue for MathEquation {
    fn render(&self, _node: &Node, fmt: &mut dyn Renderer) {
        let opts = katex::Opts::builder()
            .throw_on_error(false)
            .display_mode(self.display_mode)
            .build().unwrap();
        println!("equation: {}",self.equation);
        let katex_equation = katex::render_with_opts(&self.equation, opts).unwrap();
        fmt.text_raw(&katex_equation)
    }
}

struct MathEquationScanner;

impl InlineRule for MathEquationScanner {
    const MARKER: char = '$';

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let input = &state.src[state.pos..state.pos_max];
        if !input.starts_with("$$")  {
            return None;
        }
        const SEPARATOR_LENGTH: usize = 2;

        input[SEPARATOR_LENGTH - 1..].find("$$").map(|length| {
            let start = state.pos + SEPARATOR_LENGTH;
            let i = start + length - SEPARATOR_LENGTH + 1;
            if start > i {
                return None
            }
            let content = &state.src[start..i];
            let node = Node::new(MathEquation {
                equation: content.to_string(),
                display_mode: false
            });
            Some((node, length + SEPARATOR_LENGTH + 1))
        })?
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

#[test]
fn test_markdown_equation_katex() {
    let parser = markdown_parser();
    let rendered = parser
        .parse("here is a math equation: $$E=mc^2$$. Pretty cool, right?")
        .render();
    assert_eq!(
        "<p>here is a math equation: ".to_owned() + &katex::render("E=mc^2").unwrap() + ". Pretty cool, right?</p>\n",
        rendered
    );
}
