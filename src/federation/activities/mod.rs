use crate::database::article::DbArticle;
use crate::database::edit::{DbEdit, DbEditForm};
use crate::database::MyDataHandle;
use crate::error::Error;
use crate::federation::activities::update_local_article::UpdateLocalArticle;
use crate::federation::activities::update_remote_article::UpdateRemoteArticle;
use crate::federation::objects::instance::DbInstance;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;

pub mod accept;
pub mod create_article;
pub mod follow;
pub mod reject;
pub mod update_local_article;
pub mod update_remote_article;

pub async fn submit_article_update(
    data: &Data<MyDataHandle>,
    new_text: String,
    original_article: &DbArticle,
) -> Result<(), Error> {
    let form = DbEditForm::new(original_article, &new_text)?;
    let edit = DbEdit::create(&form, &data.db_connection)?;
    if original_article.local {
        let updated_article = {
            let mut lock = data.articles.lock().unwrap();
            let article = lock.get_mut(original_article.ap_id.inner()).unwrap();
            article.text = new_text;
            article.latest_version = edit.version.clone();
            article.clone()
        };

        UpdateLocalArticle::send(updated_article, vec![], data).await?;
    } else {
        let instance: DbInstance = ObjectId::from(original_article.instance_id.clone())
            .dereference(data)
            .await?;
        UpdateRemoteArticle::send(edit, instance, data).await?;
    }
    Ok(())
}
