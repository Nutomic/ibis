use katex;
use markdown_it::{
    Node,
    NodeValue,
    Renderer,
    parser::inline::{InlineRule, InlineState},
};

#[derive(Debug)]
struct MathEquation {
    equation: String,
    display_mode: bool,
}

impl NodeValue for MathEquation {
    fn render(&self, _node: &Node, fmt: &mut dyn Renderer) {
        let opts = katex::Opts::builder()
            .throw_on_error(false)
            .display_mode(self.display_mode)
            .build()
            .ok();
        let katex_equation = opts.and_then(|o| katex::render_with_opts(&self.equation, o).ok());
        fmt.text_raw(katex_equation.as_ref().unwrap_or(&self.equation))
    }
}

pub struct MathEquationScanner;

impl InlineRule for MathEquationScanner {
    const MARKER: char = '$';

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let input = &state.src[state.pos..state.pos_max];
        if !input.starts_with("$$") {
            return None;
        }
        let mut display_mode = false;
        if input.starts_with("$$\n") || input.starts_with("$$ ") {
            display_mode = true;
        }
        const SEPARATOR_LENGTH: usize = 2;

        input[SEPARATOR_LENGTH - 1..].find("$$").map(|length| {
            let start = state.pos + SEPARATOR_LENGTH;
            let i = start + length - SEPARATOR_LENGTH + 1;
            if start > i {
                return None;
            }
            let content = &state.src[start..i];
            let node = Node::new(MathEquation {
                equation: content.to_string(),
                display_mode,
            });
            Some((node, length + SEPARATOR_LENGTH + 1))
        })?
    }
}

#[cfg(test)]
mod test {
    use crate::markdown::render_article_markdown;

    #[test]
    #[expect(clippy::unwrap_used)]
    fn test_markdown_equation_katex() {
        let rendered =
            render_article_markdown("here is a math equation: $$E=mc^2$$. Pretty cool, right?");
        assert_eq!(
            "<p>here is a math equation: ".to_owned()
                + &katex::render("E=mc^2").unwrap()
                + ". Pretty cool, right?</p>\n",
            rendered
        );
    }
}
