use super::ApiClient;
use crate::{
    common::{
        article::{Article, SearchArticleParams},
        instance::{
            FollowInstanceParams,
            GetInstanceParams,
            Instance,
            InstanceView,
            InstanceView2,
            SiteView,
            UpdateInstanceParams,
        },
        ResolveObjectParams,
        SuccessResponse,
    },
    frontend::utils::errors::FrontendResult,
};
use http::Method;
use url::Url;

impl ApiClient {
    pub async fn get_local_instance(&self) -> FrontendResult<InstanceView2> {
        self.get("/api/v1/instance", None::<i32>).await
    }

    pub async fn get_instance(&self, params: &GetInstanceParams) -> FrontendResult<InstanceView2> {
        self.get("/api/v1/instance", Some(&params)).await
    }

    pub async fn list_instances(&self) -> FrontendResult<Vec<InstanceView>> {
        self.get("/api/v1/instance/list_views", None::<i32>).await
    }

    pub async fn update_local_instance(
        &self,
        params: &UpdateInstanceParams,
    ) -> FrontendResult<Instance> {
        self.patch("/api/v1/instance", Some(params)).await
    }

    pub async fn search(&self, params: &SearchArticleParams) -> FrontendResult<Vec<Article>> {
        self.send(Method::GET, "/api/v1/search", Some(params)).await
    }

    pub async fn resolve_instance(&self, id: Url) -> FrontendResult<Instance> {
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
    ) -> FrontendResult<Instance> {
        use crate::common::{utils::http_protocol_str, ResolveObjectParams};
        use url::Url;
        let params = ResolveObjectParams {
            id: Url::parse(&format!("{}://{}", http_protocol_str(), follow_instance))?,
        };
        let instance_resolved: Instance =
            self.get("/api/v1/instance/resolve", Some(params)).await?;

        // send follow
        let params = FollowInstanceParams {
            id: instance_resolved.id,
        };
        self.follow_instance(params).await?;
        Ok(instance_resolved)
    }
}
