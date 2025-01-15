use crate::{
    backend::{database::IbisData, utils::error::MyResult},
    common::{
        comment::{CreateCommentForm, DbComment, EditCommentForm},
        user::LocalUserView,
    },
};
use activitypub_federation::config::Data;
use axum::{Extension, Form, Json};
use axum_macros::debug_handler;

#[debug_handler]
pub(in crate::backend::api) async fn create_comment(
    user: Extension<LocalUserView>,
    data: Data<IbisData>,
    Form(create_article): Form<CreateCommentForm>,
) -> MyResult<Json<DbComment>> {
    todo!()
}

#[debug_handler]
pub(in crate::backend::api) async fn edit_comment(
    user: Extension<LocalUserView>,
    data: Data<IbisData>,
    Form(create_article): Form<EditCommentForm>,
) -> MyResult<Json<DbComment>> {
    todo!()
}
