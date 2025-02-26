use super::ApiClient;
use crate::errors::FrontendResult;
use http::Method;
use ibis_database::common::{
    ResolveObjectParams,
    SuccessResponse,
    article::{ApiConflict, Article, ArticleView, EditVersion, EditView},
    newtypes::{ArticleId, ConflictId, InstanceId, PersonId},
};
use serde::{Deserialize, Serialize};
use url::Url;

/// Should be an enum Title/Id but fails due to https://github.com/nox/serde_urlencoded/issues/66
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct GetArticleParams {
    pub title: Option<String>,
    pub domain: Option<String>,
    pub id: Option<ArticleId>,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug)]
pub struct ListArticlesParams {
    pub only_local: Option<bool>,
    pub instance_id: Option<InstanceId>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateArticleParams {
    pub title: String,
    pub text: String,
    pub summary: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EditArticleParams {
    /// Id of the article to edit
    pub article_id: ArticleId,
    /// Full, new text of the article. A diff against `previous_version` is generated on the backend
    /// side to handle conflicts.
    pub new_text: String,
    /// What was changed
    pub summary: String,
    /// The version that this edit is based on, ie [DbArticle.latest_version] or
    /// [ApiConflict.previous_version]
    pub previous_version_id: EditVersion,
    /// If you are resolving a conflict, pass the id to delete conflict from the database
    pub resolve_conflict_id: Option<ConflictId>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProtectArticleParams {
    pub article_id: ArticleId,
    pub protected: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ForkArticleParams {
    pub article_id: ArticleId,
    pub new_title: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApproveArticleParams {
    pub article_id: ArticleId,
    pub approve: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct GetEditList {
    pub article_id: Option<ArticleId>,
    pub person_id: Option<PersonId>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DeleteConflictParams {
    pub conflict_id: ConflictId,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GetConflictParams {
    pub conflict_id: ConflictId,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FollowArticleParams {
    pub id: ArticleId,
    pub follow: bool,
}

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
