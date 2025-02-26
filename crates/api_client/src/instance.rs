use super::ApiClient;
use crate::errors::FrontendResult;
use http::Method;
use ibis_database::common::{
    ResolveObjectParams,
    SuccessResponse,
    article::Article,
    instance::{Instance, InstanceView, InstanceWithArticles, SiteView},
    newtypes::InstanceId,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SearchArticleParams {
    pub query: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetInstanceParams {
    pub id: Option<InstanceId>,
    pub hostname: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FollowInstanceParams {
    pub id: InstanceId,
    pub follow: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateInstanceParams {
    pub name: Option<String>,
    pub topic: Option<String>,
}

impl ApiClient {
    pub async fn get_instance(&self, params: &GetInstanceParams) -> FrontendResult<InstanceView> {
        self.get("/api/v1/instance", Some(&params)).await
    }

    pub async fn list_instances(&self) -> FrontendResult<Vec<InstanceWithArticles>> {
        self.get("/api/v1/instance/list", None::<i32>).await
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
        id: InstanceId,
        follow: bool,
    ) -> FrontendResult<SuccessResponse> {
        let params = FollowInstanceParams { id, follow };
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
        use ibis_database::common::{ResolveObjectParams, utils::http_protocol_str};
        use url::Url;
        let params = ResolveObjectParams {
            id: Url::parse(&format!("{}://{}", http_protocol_str(), follow_instance))?,
        };
        let instance_resolved: Instance =
            self.get("/api/v1/instance/resolve", Some(params)).await?;

        // send follow
        self.follow_instance(instance_resolved.id, true).await?;
        Ok(instance_resolved)
    }
}
