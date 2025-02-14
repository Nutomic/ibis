use super::ApiClient;
use crate::{
    common::{
        comment::{CreateCommentParams, DbCommentView, EditCommentParams, MarkAsReadParams},
        newtypes::CommentId,
        SuccessResponse,
    },
    frontend::utils::errors::FrontendResult,
};

impl ApiClient {
    pub async fn create_comment(
        &self,
        params: &CreateCommentParams,
    ) -> FrontendResult<DbCommentView> {
        self.post("/api/v1/comment", Some(&params)).await
    }

    pub async fn edit_comment(&self, params: &EditCommentParams) -> FrontendResult<DbCommentView> {
        self.patch("/api/v1/comment", Some(&params)).await
    }

    pub async fn mark_comment_as_read(&self, id: CommentId) -> FrontendResult<SuccessResponse> {
        self.post(
            "/api/v1/comment/mark_as_read",
            Some(&MarkAsReadParams { id }),
        )
        .await
    }
}
