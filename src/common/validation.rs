use crate::common::DbArticle;
use anyhow::anyhow;
use anyhow::Result;

pub fn can_edit_article(article: &DbArticle, is_admin: bool) -> Result<()> {
    let err = anyhow!("Article is protected, only admins on origin instance can edit");
    if article.protected {
        if !article.local {
            return Err(err);
        }
        if !is_admin {
            return Err(err);
        }
    }
    Ok(())
}
