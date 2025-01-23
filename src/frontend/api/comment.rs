use super::ApiClient;
use crate::common::comment::{CreateCommentParams, DbCommentView, EditCommentParams};
use leptos::prelude::ServerFnError;

impl ApiClient {
    pub async fn create_comment(
        &self,
        params: &CreateCommentParams,
    ) -> Result<DbCommentView, ServerFnError> {
        self.post("/api/v1/comment", Some(&params)).await
    }

    pub async fn edit_comment(
        &self,
        params: &EditCommentParams,
    ) -> Result<DbCommentView, ServerFnError> {
        self.patch("/api/v1/comment", Some(&params)).await
    }
}
