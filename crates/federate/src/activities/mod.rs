use crate::{
    activities::article::{edit_article::EditArticle, update_article::UpdateArticle},
    objects::instance::InstanceWrapper,
    routes::AnnouncableActivities,
};
use activitypub_federation::config::Data;
use announce::AnnounceActivity;
use ibis_database::{
    common::{
        article::{Article, Edit, EditVersion},
        instance::Instance,
        newtypes::PersonId,
    },
    error::BackendResult,
    impls::{IbisContext, edit::DbEditForm},
};

pub mod announce;
pub mod article;
pub mod comment;
pub mod following;
pub mod reject;

pub async fn submit_article_update(
    new_text: String,
    summary: String,
    previous_version: EditVersion,
    article: &Article,
    creator_id: PersonId,
    context: &Data<IbisContext>,
) -> BackendResult<()> {
    let mut form = DbEditForm::new(
        article,
        creator_id,
        &new_text,
        summary,
        previous_version,
        false,
    )?;

    // insert edit to remote instance as pending, so only the creator can see it
    form.pending = !article.local;
    let edit = Edit::create(&form, context).await?;

    let local_instance: InstanceWrapper = Instance::read_local(context)?.into();
    let article_instance: InstanceWrapper = Instance::read(article.instance_id, context)?.into();
    let edit_activity =
        EditArticle::new(edit.into(), &local_instance, &article_instance, context).await?;

    if article_instance.local {
        let updated_article = Article::update_text(article.id, &new_text, context)?;

        UpdateArticle::send(updated_article.into(), &local_instance, context).await?;
        AnnounceActivity::send(AnnouncableActivities::EditArticle(edit_activity), context).await?;
    } else {
        edit_activity
            .send(&local_instance, &article_instance, context)
            .await?;
    }
    Ok(())
}
