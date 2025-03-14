use activitypub_federation::protocol::values::{MediaTypeMarkdown, MediaTypeMarkdownOrHtml};
use html2md::parse_html;
use serde::{Deserialize, Serialize};

pub mod article;
mod article_or_comment;
pub mod comment;
pub mod edit;
pub mod instance;
pub mod user;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub(crate) content: String,
    pub(crate) media_type: MediaTypeMarkdown,
}

impl Source {
    pub(crate) fn new(content: String) -> Self {
        Source {
            content,
            media_type: MediaTypeMarkdown::Markdown,
        }
    }
}

pub(crate) fn read_from_string_or_source(
    content: &str,
    media_type: &Option<MediaTypeMarkdownOrHtml>,
    source: &Option<Source>,
) -> String {
    if let Some(s) = source {
        // markdown sent by lemmy in source field
        s.content.clone()
    } else if media_type == &Some(MediaTypeMarkdownOrHtml::Markdown) {
        // markdown sent by peertube in content field
        content.to_string()
    } else {
        // otherwise, convert content html to markdown
        parse_html(content)
    }
}

pub(crate) fn read_from_string_or_source_opt(
    content: &Option<String>,
    media_type: &Option<MediaTypeMarkdownOrHtml>,
    source: &Option<Source>,
) -> Option<String> {
    content
        .as_ref()
        .map(|content| read_from_string_or_source(content, media_type, source))
}
