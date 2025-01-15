use crate::common::article::DbArticle;
use leptos::server_fn::error::ServerFnErrorErr;

pub fn can_edit_article(article: &DbArticle, is_admin: bool) -> Result<(), ServerFnErrorErr> {
    let err = ServerFnErrorErr::ServerError(
        "Article is protected, only admins on origin instance can edit".to_string(),
    );
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
