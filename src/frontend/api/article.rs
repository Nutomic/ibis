use super::ApiClient;
use crate::{
    common::{
        article::{
            ApiConflict,
            ApproveArticleParams,
            Article,
            ArticleView,
            CreateArticleParams,
            DeleteConflictParams,
            EditArticleParams,
            EditView,
            FollowArticleParams,
            ForkArticleParams,
            GetArticleParams,
            GetConflictParams,
            GetEditList,
            ListArticlesParams,
            ProtectArticleParams,
        },
        newtypes::{ArticleId, ConflictId},
        ResolveObjectParams,
        SuccessResponse,
    },
    frontend::utils::errors::FrontendResult,
};
use http::Method;
use url::Url;

impl ApiClient {
    pub async fn create_article(&self, data: &CreateArticleParams) -> FrontendResult<ArticleView> {
        self.post("/api/v1/article", Some(&data)).await
    }

    pub async fn get_article(&self, data: GetArticleParams) -> FrontendResult<ArticleView> {
        self.send(Method::GET, "/api/v1/article", Some(data)).await
    }

    pub async fn list_articles(&self, data: ListArticlesParams) -> FrontendResult<Vec<Article>> {
        self.get("/api/v1/article/list", Some(data)).await
    }

    pub async fn edit_article(
        &self,
        params: &EditArticleParams,
    ) -> FrontendResult<Option<ApiConflict>> {
        self.patch("/api/v1/article", Some(&params)).await
    }

    pub async fn fork_article(&self, params: &ForkArticleParams) -> FrontendResult<ArticleView> {
        self.post("/api/v1/article/fork", Some(params)).await
    }

    pub async fn protect_article(&self, params: &ProtectArticleParams) -> FrontendResult<Article> {
        self.post("/api/v1/article/protect", Some(params)).await
    }

    pub async fn resolve_article(&self, id: Url) -> FrontendResult<ArticleView> {
        let resolve_object = ResolveObjectParams { id };
        self.send(Method::GET, "/api/v1/article/resolve", Some(resolve_object))
            .await
    }

    pub async fn get_article_edits(&self, article_id: ArticleId) -> FrontendResult<Vec<EditView>> {
        let data = GetEditList {
            article_id: Some(article_id),
            ..Default::default()
        };
        self.send(Method::GET, "/api/v1/edit/list", Some(data))
            .await
    }

    pub async fn approve_article(
        &self,
        article_id: ArticleId,
        approve: bool,
    ) -> FrontendResult<()> {
        let params = ApproveArticleParams {
            article_id,
            approve,
        };
        self.post("/api/v1/article/approve", Some(&params)).await
    }

    pub async fn get_conflict(&self, conflict_id: ConflictId) -> FrontendResult<ApiConflict> {
        let params = GetConflictParams { conflict_id };
        self.get("/api/v1/conflict", Some(params)).await
    }

    pub async fn delete_conflict(&self, conflict_id: ConflictId) -> FrontendResult<()> {
        let params = DeleteConflictParams { conflict_id };
        self.send(Method::DELETE, "/api/v1/conflict", Some(params))
            .await
    }

    pub async fn follow_article(
        &self,
        id: ArticleId,
        follow: bool,
    ) -> FrontendResult<SuccessResponse> {
        let params = FollowArticleParams { id, follow };
        self.post("/api/v1/article/follow", Some(params)).await
    }

    #[cfg(debug_assertions)]
    pub async fn edit_article_without_conflict(
        &self,
        params: &EditArticleParams,
    ) -> Option<ArticleView> {
        let edit_res = self
            .edit_article(params)
            .await
            .map_err(|e| log::error!("edit failed {e}"))
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
