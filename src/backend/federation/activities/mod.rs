use crate::{
    backend::{
        database::{edit::DbEditForm, IbisContext},
        federation::activities::{
            update_local_article::UpdateLocalArticle,
            update_remote_article::UpdateRemoteArticle,
        },
        utils::error::Error,
    },
    common::{
        article::{DbArticle, DbEdit, EditVersion},
        instance::DbInstance,
        newtypes::PersonId,
    },
};
use activitypub_federation::config::Data;

pub mod accept;
pub mod announce;
pub mod comment;
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
    creator_id: PersonId,
    context: &Data<IbisContext>,
) -> Result<(), Error> {
    let mut form = DbEditForm::new(
        original_article,
        creator_id,
        &new_text,
        summary,
        previous_version,
        false,
    )?;
    if original_article.local {
        let edit = DbEdit::create(&form, context)?;
        let updated_article = DbArticle::update_text(edit.article_id, &new_text, context)?;

        UpdateLocalArticle::send(updated_article, vec![], context).await?;
    } else {
        // insert edit as pending, so only the creator can see it
        form.pending = true;
        let edit = DbEdit::create(&form, context)?;
        let instance = DbInstance::read(original_article.instance_id, context)?;
        UpdateRemoteArticle::send(edit, instance, context).await?;
    }
    Ok(())
}
