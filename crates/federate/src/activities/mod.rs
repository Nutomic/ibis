use crate::activities::{
    update_local_article::UpdateLocalArticle,
    update_remote_article::UpdateRemoteArticle,
};
use activitypub_federation::config::Data;
use ibis_database::{
    common::{
        article::{Article, Edit, EditVersion},
        instance::Instance,
        newtypes::PersonId,
    },
    error::BackendResult,
    impls::{IbisContext, edit::DbEditForm},
};

pub mod accept;
pub mod announce;
pub mod comment;
pub mod create_article;
pub mod follow;
pub mod reject;
pub mod undo_follow;
pub mod update_local_article;
pub mod update_remote_article;

pub async fn submit_article_update(
    new_text: String,
    summary: String,
    previous_version: EditVersion,
    original_article: &Article,
    creator_id: PersonId,
    context: &Data<IbisContext>,
) -> BackendResult<()> {
    let mut form = DbEditForm::new(
        original_article,
        creator_id,
        &new_text,
        summary,
        previous_version,
        false,
    )?;
    if original_article.local {
        let edit = Edit::create(&form, context)?;
        let updated_article = Article::update_text(edit.article_id, &new_text, context)?;

        UpdateLocalArticle::send(updated_article.into(), vec![], context).await?;
    } else {
        // insert edit as pending, so only the creator can see it
        form.pending = true;
        let edit = Edit::create(&form, context)?;
        let instance = Instance::read(original_article.instance_id, context)?;
        UpdateRemoteArticle::send(edit.into(), instance, context).await?;
    }
    Ok(())
}
