use crate::common::{DbArticle, MAIN_PAGE_NAME};
use anyhow::anyhow;
use anyhow::Result;

pub fn can_edit_article(article: &DbArticle, is_admin: bool) -> Result<()> {
    if article.title == MAIN_PAGE_NAME {
        if !article.local {
            return Err(anyhow!("Cannot edit main page of remote instance"));
        }
        if article.local && !is_admin {
            return Err(anyhow!("Only admin can edit main page"));
        }
    }
    Ok(())
}
