use crate::database::{DatabaseHandle, DbConflict};
use crate::error::MyResult;
use crate::federation::activities::create_article::CreateArticle;
use crate::federation::activities::submit_article_update;
use crate::federation::objects::article::DbArticle;
use crate::federation::objects::edit::EditVersion;
use crate::federation::objects::instance::DbInstance;
use crate::utils::generate_article_version;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use anyhow::anyhow;
use axum::extract::Query;
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use axum_macros::debug_handler;
use diffy::create_patch;
use futures::future::try_join_all;
use rand::random;
use serde::{Deserialize, Serialize};
use url::Url;

pub fn api_routes() -> Router {
    Router::new()
        .route(
            "/article",
            get(get_article).post(create_article).patch(edit_article),
        )
        .route("/article/fork", post(fork_article))
        .route("/edit_conflicts", get(edit_conflicts))
        .route("/resolve_instance", get(resolve_instance))
        .route("/resolve_article", get(resolve_article))
        .route("/instance", get(get_local_instance))
        .route("/instance/follow", post(follow_instance))
        .route("/search", get(search_article))
}

#[derive(Deserialize, Serialize)]
pub struct CreateArticleData {
    pub title: String,
}

#[debug_handler]
async fn create_article(
    data: Data<DatabaseHandle>,
    Form(create_article): Form<CreateArticleData>,
) -> MyResult<Json<DbArticle>> {
    {
        let articles = data.articles.lock().unwrap();
        let title_exists = articles
            .iter()
            .any(|a| a.1.local && a.1.title == create_article.title);
        if title_exists {
            return Err(anyhow!("A local article with this title already exists").into());
        }
    }

    let local_instance_id = data.local_instance().ap_id;
    let ap_id = ObjectId::parse(&format!(
        "http://{}:{}/article/{}",
        local_instance_id.inner().domain().unwrap(),
        local_instance_id.inner().port().unwrap(),
        create_article.title
    ))?;
    let article = DbArticle {
        title: create_article.title,
        text: String::new(),
        ap_id,
        latest_version: EditVersion::default(),
        edits: vec![],
        instance: local_instance_id,
        local: true,
    };
    {
        let mut articles = data.articles.lock().unwrap();
        articles.insert(article.ap_id.inner().clone(), article.clone());
    }

    CreateArticle::send_to_followers(article.clone(), &data).await?;

    Ok(Json(article))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EditArticleData {
    pub ap_id: ObjectId<DbArticle>,
    pub new_text: String,
    pub previous_version: EditVersion,
    pub resolve_conflict_id: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ApiConflict {
    pub id: i32,
    pub three_way_merge: String,
    pub article_id: ObjectId<DbArticle>,
    pub previous_version: EditVersion,
}

#[debug_handler]
async fn edit_article(
    data: Data<DatabaseHandle>,
    Form(edit_form): Form<EditArticleData>,
) -> MyResult<Json<Option<ApiConflict>>> {
    // resolve conflict if any
    if let Some(resolve_conflict_id) = &edit_form.resolve_conflict_id {
        let mut lock = data.conflicts.lock().unwrap();
        if !lock.iter().any(|c| &c.id == resolve_conflict_id) {
            return Err(anyhow!("invalid resolve conflict"))?;
        }
        lock.retain(|c| &c.id != resolve_conflict_id);
    }
    let original_article = {
        let lock = data.articles.lock().unwrap();
        let article = lock.get(edit_form.ap_id.inner()).unwrap();
        article.clone()
    };

    if edit_form.previous_version == original_article.latest_version {
        // No intermediate changes, simply submit new version
        submit_article_update(&data, edit_form.new_text.clone(), &original_article).await?;
        Ok(Json(None))
    } else {
        // There have been other changes since this edit was initiated. Get the common ancestor
        // version and generate a diff to find out what exactly has changed.
        let ancestor =
            generate_article_version(&original_article.edits, &edit_form.previous_version)?;
        let patch = create_patch(&ancestor, &edit_form.new_text);

        let db_conflict = DbConflict {
            id: random(),
            diff: patch.to_string(),
            article_id: original_article.ap_id.clone(),
            previous_version: edit_form.previous_version,
        };
        {
            let mut lock = data.conflicts.lock().unwrap();
            lock.push(db_conflict.clone());
        }
        Ok(Json(db_conflict.to_api_conflict(&data).await?))
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GetArticleData {
    pub ap_id: ObjectId<DbArticle>,
}

#[debug_handler]
async fn get_article(
    Query(query): Query<GetArticleData>,
    data: Data<DatabaseHandle>,
) -> MyResult<Json<DbArticle>> {
    let articles = data.articles.lock().unwrap();
    let article = articles
        .iter()
        .find(|a| a.1.ap_id == query.ap_id)
        .ok_or(anyhow!("not found"))?
        .1
        .clone();
    Ok(Json(article))
}

#[derive(Deserialize, Serialize)]
pub struct ResolveObject {
    pub id: Url,
}

#[debug_handler]
async fn resolve_instance(
    Query(query): Query<ResolveObject>,
    data: Data<DatabaseHandle>,
) -> MyResult<Json<DbInstance>> {
    let instance: DbInstance = ObjectId::from(query.id).dereference(&data).await?;
    Ok(Json(instance))
}

#[debug_handler]
async fn resolve_article(
    Query(query): Query<ResolveObject>,
    data: Data<DatabaseHandle>,
) -> MyResult<Json<DbArticle>> {
    let article: DbArticle = ObjectId::from(query.id).dereference(&data).await?;
    Ok(Json(article))
}

#[debug_handler]
async fn get_local_instance(data: Data<DatabaseHandle>) -> MyResult<Json<DbInstance>> {
    Ok(Json(data.local_instance()))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FollowInstance {
    pub instance_id: ObjectId<DbInstance>,
}

#[debug_handler]
async fn follow_instance(
    data: Data<DatabaseHandle>,
    Form(query): Form<FollowInstance>,
) -> MyResult<()> {
    let instance = query.instance_id.dereference(&data).await?;
    data.local_instance().follow(&instance, &data).await?;
    Ok(())
}

#[debug_handler]
async fn edit_conflicts(data: Data<DatabaseHandle>) -> MyResult<Json<Vec<ApiConflict>>> {
    let conflicts = { data.conflicts.lock().unwrap().to_vec() };
    let conflicts: Vec<ApiConflict> = try_join_all(conflicts.into_iter().map(|c| {
        let data = data.reset_request_count();
        async move { c.to_api_conflict(&data).await }
    }))
    .await?
    .into_iter()
    .flatten()
    .collect();
    Ok(Json(conflicts))
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SearchArticleData {
    pub title: String,
}

#[debug_handler]
async fn search_article(
    Query(query): Query<SearchArticleData>,
    data: Data<DatabaseHandle>,
) -> MyResult<Json<Vec<DbArticle>>> {
    let articles = data.articles.lock().unwrap();
    let article = articles
        .iter()
        .filter(|a| a.1.title == query.title)
        .map(|a| a.1)
        .cloned()
        .collect();
    Ok(Json(article))
}

#[derive(Deserialize, Serialize)]
pub struct ForkArticleData {
    // TODO: could add optional param new_title so there is no problem with title collision
    //       in case local article with same title exists
    pub ap_id: ObjectId<DbArticle>,
}

#[debug_handler]
async fn fork_article(
    data: Data<DatabaseHandle>,
    Form(fork_form): Form<ForkArticleData>,
) -> MyResult<Json<DbArticle>> {
    let article = {
        let lock = data.articles.lock().unwrap();
        let article = lock.get(fork_form.ap_id.inner()).unwrap();
        article.clone()
    };
    if article.local {
        return Err(anyhow!("Cannot fork local article because there cant be multiple local articles with same title").into());
    }

    let original_article = {
        let lock = data.articles.lock().unwrap();
        lock.get(fork_form.ap_id.inner())
            .expect("article exists")
            .clone()
    };

    let local_instance_id = data.local_instance().ap_id;
    let ap_id = ObjectId::parse(&format!(
        "http://{}:{}/article/{}",
        local_instance_id.inner().domain().unwrap(),
        local_instance_id.inner().port().unwrap(),
        original_article.title
    ))?;
    let forked_article = DbArticle {
        title: original_article.title.clone(),
        text: original_article.text.clone(),
        ap_id,
        latest_version: original_article.latest_version.clone(),
        edits: original_article.edits.clone(),
        instance: local_instance_id,
        local: true,
    };
    {
        let mut articles = data.articles.lock().unwrap();
        articles.insert(forked_article.ap_id.inner().clone(), forked_article.clone());
    }

    CreateArticle::send_to_followers(forked_article.clone(), &data).await?;

    Ok(Json(forked_article))
}
