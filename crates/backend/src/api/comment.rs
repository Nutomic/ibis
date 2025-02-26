use super::UserExt;
use activitypub_federation::config::Data;
use anyhow::anyhow;
use axum::{Form, Json};
use axum_macros::debug_handler;
use chrono::Utc;
use ibis_api_client::comment::{CreateCommentParams, EditCommentParams};
use ibis_database::{
    common::{
        comment::{Comment, CommentView},
        utils::http_protocol_str,
    },
    error::BackendResult,
    impls::{
        IbisContext,
        comment::{DbCommentInsertForm, DbCommentUpdateForm},
    },
};
use ibis_federate::{
    activities::comment::{
        create_or_update_comment::CreateOrUpdateComment,
        delete_comment::DeleteComment,
        undo_delete_comment::UndoDeleteComment,
    },
    objects::comment::CommentWrapper,
    validate::{validate_comment_max_depth, validate_not_empty},
};
use url::Url;

#[debug_handler]
pub(crate) async fn create_comment(
    user: UserExt,
    context: Data<IbisContext>,
    Form(params): Form<CreateCommentParams>,
) -> BackendResult<Json<CommentView>> {
    validate_not_empty(&params.content)?;
    let mut depth = 0;
    if let Some(parent_id) = params.parent_id {
        let parent = Comment::read(parent_id, &context)?;
        if parent.deleted {
            return Err(anyhow!("Cant reply to deleted comment").into());
        }
        if parent.article_id != params.article_id {
            return Err(anyhow!("Invalid article_id/parent_id combination").into());
        }
        depth = parent.depth + 1;
        validate_comment_max_depth(depth)?;
    }
    let form = DbCommentInsertForm {
        creator_id: user.person.id,
        article_id: params.article_id,
        parent_id: params.parent_id,
        content: params.content,
        depth,
        ap_id: None,
        local: true,
        deleted: false,
        published: Utc::now(),
        updated: None,
    };
    let comment = Comment::create(form, &context)?;

    // Set the ap_id which contains db id (so it is not know before inserting)
    let proto = http_protocol_str();
    let ap_id = Url::parse(&format!(
        "{}://{}/comment/{}",
        proto,
        context.domain(),
        comment.id.0
    ))?
    .into();
    let form = DbCommentUpdateForm {
        ap_id: Some(ap_id),
        ..Default::default()
    };
    let comment = Comment::update(form, comment.id, &context)?;

    CreateOrUpdateComment::send(&comment.comment.clone().into(), &context).await?;

    Ok(Json(comment))
}

#[debug_handler]
pub(crate) async fn edit_comment(
    user: UserExt,
    context: Data<IbisContext>,
    Form(params): Form<EditCommentParams>,
) -> BackendResult<Json<CommentView>> {
    if let Some(content) = &params.content {
        validate_not_empty(content)?;
    }
    if params.content.is_none() && params.deleted.is_none() {
        return Err(anyhow!("Edit has no parameters").into());
    }
    let orig_comment = Comment::read(params.id, &context)?;
    if orig_comment.creator_id != user.person.id {
        return Err(anyhow!("Cannot edit comment created by another user").into());
    }
    let form = DbCommentUpdateForm {
        content: params.content,
        deleted: params.deleted,
        updated: Some(Utc::now()),
        ..Default::default()
    };
    let comment = Comment::update(form, params.id, &context)?;

    let apub_comment: CommentWrapper = comment.comment.clone().into();
    // federate
    if orig_comment.content != comment.comment.content {
        CreateOrUpdateComment::send(&apub_comment, &context).await?;
    }
    if !orig_comment.deleted && comment.comment.deleted {
        DeleteComment::send(&apub_comment, &context).await?;
    }
    if orig_comment.deleted && !comment.comment.deleted {
        UndoDeleteComment::send(&apub_comment, &context).await?;
    }

    Ok(Json(comment))
}
