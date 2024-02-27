use crate::backend::database::edit::DbEditForm;
use crate::backend::database::IbisData;
use crate::backend::error::Error;
use crate::backend::federation::activities::update_local_article::UpdateLocalArticle;
use crate::backend::federation::activities::update_remote_article::UpdateRemoteArticle;
use crate::common::DbInstance;
use crate::common::EditVersion;
use crate::common::{DbArticle, DbEdit};
use activitypub_federation::config::Data;
use chrono::Utc;

pub mod accept;
pub mod create_article;
pub mod follow;
pub mod reject;
pub mod update_local_article;
pub mod update_remote_article;

pub async fn submit_article_update(
    new_text: String,
    summary: String,
    previous_version: EditVersion,
    original_article: &DbArticle,
    creator_id: i32,
    data: &Data<IbisData>,
) -> Result<(), Error> {
    let form = DbEditForm::new(
        original_article,
        creator_id,
        &new_text,
        summary,
        previous_version,
    )?;
    if original_article.local {
        let edit = DbEdit::create(&form, data)?;
        let updated_article = DbArticle::update_text(edit.article_id, &new_text, data)?;

        UpdateLocalArticle::send(updated_article, vec![], data).await?;
    } else {
        // dont insert edit into db, might be invalid in case of conflict
        let edit = DbEdit {
            id: -1,
            creator_id,
            hash: form.hash,
            ap_id: form.ap_id,
            diff: form.diff,
            summary: form.summary,
            article_id: form.article_id,
            previous_version_id: form.previous_version_id,
            created: Utc::now(),
        };
        let instance = DbInstance::read(original_article.instance_id, data)?;
        UpdateRemoteArticle::send(edit, instance, data).await?;
    }
    Ok(())
}
