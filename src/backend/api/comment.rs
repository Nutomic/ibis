use super::UserExt;
use crate::{
    backend::{
        database::{
            comment::{DbCommentInsertForm, DbCommentUpdateForm},
            IbisContext,
        },
        federation::activities::comment::{
            create_or_update_comment::CreateOrUpdateComment,
            delete_comment::DeleteComment,
            undo_delete_comment::UndoDeleteComment,
        },
        utils::{
            error::BackendResult,
            validate::{validate_comment_max_depth, validate_not_empty},
        },
    },
    common::{
        comment::{Comment, CommentView, CreateCommentParams, EditCommentParams},
        utils::http_protocol_str,
    },
};
use activitypub_federation::config::Data;
use anyhow::anyhow;
use axum::{Form, Json};
use axum_macros::debug_handler;
use chrono::Utc;

#[debug_handler]
pub(in crate::backend::api) async fn create_comment(
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
    let ap_id = format!("{}://{}/comment/{}", proto, context.domain(), comment.id.0).parse()?;
    let form = DbCommentUpdateForm {
        ap_id: Some(ap_id),
        ..Default::default()
    };
    let comment = Comment::update(form, comment.id, &context)?;

    CreateOrUpdateComment::send(&comment.comment, &context).await?;

    Ok(Json(comment))
}

#[debug_handler]
pub(in crate::backend::api) async fn edit_comment(
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

    // federate
    if orig_comment.content != comment.comment.content {
        CreateOrUpdateComment::send(&comment.comment, &context).await?;
    }
    if !orig_comment.deleted && comment.comment.deleted {
        DeleteComment::send(&comment.comment, &context).await?;
    }
    if orig_comment.deleted && !comment.comment.deleted {
        UndoDeleteComment::send(&comment.comment, &context).await?;
    }

    Ok(Json(comment))
}
