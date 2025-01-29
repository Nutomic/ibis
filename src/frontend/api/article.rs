use super::{result_to_option, ApiClient};
use crate::{
    common::{
        article::{
            ApiConflict, ApproveArticleParams, CreateArticleParams, DbArticle, DbArticleView,
            DeleteConflictParams, EditArticleParams, EditView, ForkArticleParams, GetArticleParams,
            GetEditList, ListArticlesParams, ProtectArticleParams,
        },
        newtypes::{ArticleId, ConflictId},
        ResolveObjectParams,
    },
    frontend::utils::errors::{FrontendError, FrontendResult},
};
use http::Method;
use leptos::prelude::ServerFnError;
use log::error;
use url::Url;

impl ApiClient {
    pub async fn create_article(
        &self,
        data: &CreateArticleParams,
    ) -> Result<DbArticleView, ServerFnError> {
        self.post("/api/v1/article", Some(&data)).await
    }

    pub async fn get_article(&self, data: GetArticleParams) -> FrontendResult<DbArticleView> {
        //self.get("/api/v1/article", Some(data)).await;
        Err(FrontendError("test".to_string()))
    }

    pub async fn list_articles(&self, data: ListArticlesParams) -> Option<Vec<DbArticle>> {
        Some(self.get("/api/v1/article/list", Some(data)).await.unwrap())
    }

    pub async fn edit_article(
        &self,
        params: &EditArticleParams,
    ) -> Result<Option<ApiConflict>, ServerFnError> {
        self.patch("/api/v1/article", Some(&params)).await
    }

    pub async fn fork_article(
        &self,
        params: &ForkArticleParams,
    ) -> Result<DbArticleView, ServerFnError> {
        self.post("/api/v1/article/fork", Some(params)).await
    }

    pub async fn protect_article(
        &self,
        params: &ProtectArticleParams,
    ) -> Result<DbArticle, ServerFnError> {
        self.post("/api/v1/article/protect", Some(params)).await
    }

    pub async fn resolve_article(&self, id: Url) -> Result<DbArticleView, ServerFnError> {
        let resolve_object = ResolveObjectParams { id };
        self.send(Method::GET, "/api/v1/article/resolve", Some(resolve_object))
            .await
    }

    pub async fn get_article_edits(&self, article_id: ArticleId) -> Option<Vec<EditView>> {
        let data = GetEditList {
            article_id: Some(article_id),
            ..Default::default()
        };
        self.get("/api/v1/edit/list", Some(data)).await
    }

    pub async fn approve_article(&self, article_id: ArticleId, approve: bool) -> Option<()> {
        let params = ApproveArticleParams {
            article_id,
            approve,
        };
        result_to_option(self.post("/api/v1/article/approve", Some(&params)).await)
    }

    pub async fn delete_conflict(&self, conflict_id: ConflictId) -> Option<()> {
        let params = DeleteConflictParams { conflict_id };
        result_to_option(
            self.send(Method::DELETE, "/api/v1/conflict", Some(params))
                .await,
        )
    }

    #[cfg(debug_assertions)]
    pub async fn edit_article_without_conflict(
        &self,
        params: &EditArticleParams,
    ) -> Option<DbArticleView> {
        let edit_res = self
            .edit_article(params)
            .await
            .map_err(|e| error!("edit failed {e}"))
            .ok()?;
        assert_eq!(None, edit_res);

        self.get_article(GetArticleParams {
            title: None,
            domain: None,
            id: Some(params.article_id),
        })
        .await
        .ok()
    }
}
