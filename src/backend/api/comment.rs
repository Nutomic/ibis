use axum::{Extension, Json};
use axum_macros::debug_handler;
use crate::{backend::utils::error::MyResult, common::{DbComment, LocalUserView}};

#[debug_handler]
pub(in crate::backend::api) async fn create_comment(
    user: Extension<LocalUserView>,
    data: Data<IbisData>,
    Form(create_article): Form<CreateArticleForm>,
) -> MyResult<Json<DbComment>> {
    todo!()
}

#[debug_handler]
pub(in crate::backend::api) async fn edit_comment(
    user: Extension<LocalUserView>,
    data: Data<IbisData>,
    Form(create_article): Form<CreateArticleForm>,
) -> MyResult<Json<ArticleView>> {
    todo!()
}