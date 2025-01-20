use crate::{
    backend::{
        database::{
            comment::{DbCommentInsertForm, DbCommentUpdateForm},
            IbisData,
        },
        federation::activities::comment::{
            create_or_update_comment::CreateOrUpdateComment,
            delete_comment::DeleteComment,
            undo_delete_comment::UndoDeleteComment,
        },
        utils::{error::MyResult, validate::validate_comment_max_depth},
    },
    common::{
        comment::{CreateCommentForm, DbComment, DbCommentView, EditCommentForm},
        user::LocalUserView,
        utils::http_protocol_str,
    },
};
use activitypub_federation::config::Data;
use anyhow::anyhow;
use axum::{Extension, Form, Json};
use axum_macros::debug_handler;
use chrono::Utc;

#[debug_handler]
pub(in crate::backend::api) async fn create_comment(
    user: Extension<LocalUserView>,
    data: Data<IbisData>,
    Form(create_comment): Form<CreateCommentForm>,
) -> MyResult<Json<DbCommentView>> {
    let mut depth = 0;
    if let Some(parent_id) = create_comment.parent_id {
        let parent = DbComment::read(parent_id, &data)?;
        if parent.article_id != create_comment.article_id {
            return Err(anyhow!("Invalid article_id/parent_id combination").into());
        }
        depth = parent.depth + 1;
        validate_comment_max_depth(depth)?;
    }
    let form = DbCommentInsertForm {
        creator_id: user.person.id,
        article_id: create_comment.article_id,
        parent_id: create_comment.parent_id,
        content: create_comment.content,
        depth,
        ap_id: None,
        local: true,
        deleted: false,
        published: Utc::now(),
        updated: None,
    };
    let comment = DbComment::create(form, &data)?;

    // Set the ap_id which contains db id (so it is not know before inserting)
    let proto = http_protocol_str();
    let ap_id = format!("{}://{}/comment/{}", proto, data.domain(), comment.id.0).parse()?;
    let form = DbCommentUpdateForm {
        ap_id: Some(ap_id),
        ..Default::default()
    };
    let comment = DbComment::update(form, comment.id, &data)?;

    CreateOrUpdateComment::send(&comment.comment, &data).await?;

    Ok(Json(comment))
}

#[debug_handler]
pub(in crate::backend::api) async fn edit_comment(
    user: Extension<LocalUserView>,
    data: Data<IbisData>,
    Form(params): Form<EditCommentForm>,
) -> MyResult<Json<DbCommentView>> {
    if params.content.is_none() && params.deleted.is_none() {
        return Err(anyhow!("Edit has no parameters").into());
    }
    let orig_comment = DbComment::read(params.id, &data)?;
    if orig_comment.creator_id != user.person.id {
        return Err(anyhow!("Cannot edit comment created by another user").into());
    }
    let form = DbCommentUpdateForm {
        content: params.content,
        deleted: params.deleted,
        updated: Some(Utc::now()),
        ..Default::default()
    };
    let comment = DbComment::update(form, params.id, &data)?;

    // federate
    if orig_comment.content != comment.comment.content {
        CreateOrUpdateComment::send(&comment.comment, &data).await?;
    }
    if !orig_comment.deleted && comment.comment.deleted {
        DeleteComment::send(&comment.comment, &data).await?;
    }
    if orig_comment.deleted && !comment.comment.deleted {
        UndoDeleteComment::send(&comment.comment, &data).await?;
    }

    Ok(Json(comment))
}
