use crate::{
    common::{
        newtypes::{ArticleNotifId, CommentId},
        notifications::{ArticleNotifMarkAsReadParams, MarkAsReadParams, Notification},
        SuccessResponse,
    },
    frontend::utils::errors::FrontendResult,
};

use super::ApiClient;

impl ApiClient {
    pub async fn notifications_list(&self) -> FrontendResult<Vec<Notification>> {
        self.get("/api/v1/user/notifications/list", None::<()>)
            .await
    }

    pub async fn notifications_count(&self) -> FrontendResult<usize> {
        self.get("/api/v1/user/notifications/count", None::<()>)
            .await
    }

    pub async fn mark_comment_as_read(&self, id: CommentId) -> FrontendResult<SuccessResponse> {
        self.post(
            "/api/v1/comment/mark_as_read",
            Some(&MarkAsReadParams { id }),
        )
        .await
    }

    pub async fn article_notif_mark_as_read(
        &self,
        id: ArticleNotifId,
    ) -> FrontendResult<SuccessResponse> {
        let params = ArticleNotifMarkAsReadParams { id };
        self.post("/api/v1/user/notifications/mark_as_read", Some(params))
            .await
    }
}
