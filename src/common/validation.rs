use crate::common::{DbArticle, MAIN_PAGE_NAME};
use anyhow::anyhow;
use anyhow::Result;

pub fn can_edit_article(article: &DbArticle, is_admin: bool) -> Result<()> {
    if article.local && article.title == MAIN_PAGE_NAME && !is_admin {
        return Err(anyhow!("Only admin can edit main page"));
    }
    Ok(())
}
