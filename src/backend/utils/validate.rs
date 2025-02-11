use super::error::BackendResult;
use anyhow::anyhow;
use regex::Regex;
use std::sync::LazyLock;

pub fn validate_article_title(title: &str) -> BackendResult<String> {
    #[expect(clippy::expect_used)]
    static TITLE_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_]{3,100}$").expect("compile regex"));
    let title = title.replace(' ', "_");
    if !TITLE_REGEX.is_match(&title) {
        return Err(anyhow!("Invalid title").into());
    }
    Ok(title)
}

pub fn validate_user_name(name: &str) -> BackendResult<()> {
    #[allow(clippy::expect_used)]
    static VALID_ACTOR_NAME_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_]{3,20}$").expect("compile regex"));

    if VALID_ACTOR_NAME_REGEX.is_match(name) {
        Ok(())
    } else {
        Err(anyhow!("Invalid username").into())
    }
}

pub fn validate_display_name(name: &Option<String>) -> BackendResult<()> {
    if let Some(name) = name {
        if name.contains('@') || name.len() < 3 || name.len() > 20 {
            return Err(anyhow!("Invalid displayname").into());
        }
    }
    Ok(())
}

pub fn validate_comment_max_depth(depth: i32) -> BackendResult<()> {
    if depth > 50 {
        return Err(anyhow!("Max comment depth reached").into());
    }
    Ok(())
}

pub fn validate_not_empty(text: &str) -> BackendResult<()> {
    if text.trim().len() < 2 {
        return Err(anyhow!("Empty text submitted").into());
    }
    Ok(())
}

#[test]
#[expect(clippy::unwrap_used)]
fn test_validate_article_title() {
    assert_eq!(
        validate_article_title("With space 123").unwrap(),
        "With_space_123"
    );
    assert!(validate_article_title(&"long".to_string().repeat(100)).is_err());
    assert!(validate_article_title("a").is_err());
}
