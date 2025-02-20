use super::ApiClient;
use crate::{
    common::comment::{CommentView, CreateCommentParams, EditCommentParams},
    frontend::utils::errors::FrontendResult,
};

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
