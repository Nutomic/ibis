use super::ApiClient;
use crate::{
    common::{
        article::{DbArticle, SearchArticleParams},
        instance::{
            DbInstance,
            FollowInstanceParams,
            GetInstanceParams,
            InstanceView,
            SiteView,
            UpdateInstanceParams,
        },
        Notification,
        ResolveObjectParams,
        SuccessResponse,
    },
    frontend::utils::errors::FrontendResult,
};
use http::Method;
use url::Url;

impl ApiClient {
    pub async fn get_local_instance(&self) -> FrontendResult<InstanceView> {
        self.get("/api/v1/instance", None::<i32>).await
    }

    pub async fn get_instance(&self, params: &GetInstanceParams) -> FrontendResult<InstanceView> {
        self.get("/api/v1/instance", Some(&params)).await
    }

    pub async fn list_instances(&self) -> FrontendResult<Vec<DbInstance>> {
        self.get("/api/v1/instance/list", None::<i32>).await
    }

    pub async fn update_local_instance(
        &self,
        params: &UpdateInstanceParams,
    ) -> FrontendResult<DbInstance> {
        self.patch("/api/v1/instance", Some(params)).await
    }

    pub async fn notifications_list(&self) -> FrontendResult<Vec<Notification>> {
        self.get("/api/v1/user/notifications/list", None::<()>)
            .await
    }

    pub async fn notifications_count(&self) -> FrontendResult<usize> {
        self.get("/api/v1/user/notifications/count", None::<()>)
            .await
    }
    pub async fn search(&self, params: &SearchArticleParams) -> FrontendResult<Vec<DbArticle>> {
        self.send(Method::GET, "/api/v1/search", Some(params)).await
    }

    pub async fn resolve_instance(&self, id: Url) -> FrontendResult<DbInstance> {
        let resolve_object = ResolveObjectParams { id };
        self.send(
            Method::GET,
            "/api/v1/instance/resolve",
            Some(resolve_object),
        )
        .await
    }

    pub async fn follow_instance(
        &self,
        params: FollowInstanceParams,
    ) -> FrontendResult<SuccessResponse> {
        self.post("/api/v1/instance/follow", Some(params)).await
    }

    pub async fn site(&self) -> FrontendResult<SiteView> {
        self.get("/api/v1/site", None::<()>).await
    }

    #[cfg(debug_assertions)]
    pub async fn follow_instance_with_resolve(
        &self,
        follow_instance: &str,
    ) -> FrontendResult<DbInstance> {
        use crate::common::{utils::http_protocol_str, ResolveObjectParams};
        use url::Url;
        let params = ResolveObjectParams {
            id: Url::parse(&format!("{}://{}", http_protocol_str(), follow_instance))?,
        };
        let instance_resolved: DbInstance =
            self.get("/api/v1/instance/resolve", Some(params)).await?;

        // send follow
        let params = FollowInstanceParams {
            id: instance_resolved.id,
        };
        self.follow_instance(params).await?;
        Ok(instance_resolved)
    }
}
