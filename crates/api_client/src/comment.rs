use super::ApiClient;
use crate::errors::FrontendResult;
use ibis_database::common::{
    comment::CommentView,
    newtypes::{ArticleId, CommentId},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateCommentParams {
    pub content: String,
    pub article_id: ArticleId,
    pub parent_id: Option<CommentId>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EditCommentParams {
    pub id: CommentId,
    pub content: Option<String>,
    pub deleted: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DeleteCommentParams {
    pub id: CommentId,
}

impl ApiClient {
    pub async fn create_comment(
        &self,
        params: &CreateCommentParams,
    ) -> FrontendResult<CommentView> {
        self.post("/api/v1/comment", Some(&params)).await
    }

    pub async fn edit_comment(&self, params: &EditCommentParams) -> FrontendResult<CommentView> {
        self.patch("/api/v1/comment", Some(&params)).await
    }
}
