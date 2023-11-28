use crate::database::DatabaseHandle;
use crate::error::Error;
use crate::federation::activities::update_local_article::UpdateLocalArticle;
use crate::federation::activities::update_remote_article::UpdateRemoteArticle;
use crate::federation::objects::article::DbArticle;
use crate::federation::objects::edit::DbEdit;
use activitypub_federation::config::Data;

pub mod accept;
pub mod create_article;
pub mod follow;
pub mod reject;
pub mod update_local_article;
pub mod update_remote_article;

pub async fn submit_article_update(
    data: &Data<DatabaseHandle>,
    new_text: String,
    original_article: &DbArticle,
) -> Result<(), Error> {
    let edit = DbEdit::new(original_article, &new_text)?;
    if original_article.local {
        let updated_article = {
            let mut lock = data.articles.lock().unwrap();
            let article = lock.get_mut(original_article.ap_id.inner()).unwrap();
            article.text = new_text;
            article.latest_version = edit.version.clone();
            article.edits.push(edit.clone());
            article.clone()
        };

        UpdateLocalArticle::send(updated_article, vec![], data).await?;
    } else {
        UpdateRemoteArticle::send(
            edit,
            original_article.instance.dereference(data).await?,
            data,
        )
        .await?;
    }
    Ok(())
}
