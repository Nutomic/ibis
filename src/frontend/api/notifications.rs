use super::ApiClient;
use crate::frontend::utils::errors::FrontendResult;
use ibis_database::common::{
    newtypes::ArticleNotifId,
    notifications::{ApiNotification, ArticleNotifMarkAsReadParams},
    SuccessResponse,
};

impl ApiClient {
    pub async fn notifications_list(&self) -> FrontendResult<Vec<ApiNotification>> {
        self.get("/api/v1/user/notifications/list", None::<()>)
            .await
    }

    pub async fn notifications_count(&self) -> FrontendResult<usize> {
        self.get("/api/v1/user/notifications/count", None::<()>)
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
