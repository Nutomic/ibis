use super::ApiClient;
use crate::errors::FrontendResult;
use ibis_database::common::{
    SuccessResponse,
    newtypes::NotificationId,
    notifications::ApiNotification,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MarkAsReadParams {
    pub id: NotificationId,
}

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
        id: NotificationId,
    ) -> FrontendResult<SuccessResponse> {
        let params = MarkAsReadParams { id };
        self.post("/api/v1/user/notifications/mark_as_read", Some(params))
            .await
    }
}
