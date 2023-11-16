use crate::database::DatabaseHandle;

use crate::error::MyResult;
use crate::federation::objects::article::DbArticle;
use crate::federation::objects::instance::DbInstance;
use crate::utils::generate_object_id;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use anyhow::anyhow;
use axum::extract::Query;
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use url::Url;

pub fn api_routes() -> Router {
    Router::new()
        .route("/article", get(get_article).post(create_article))
        .route("/resolve_object", get(resolve_object))
        .route("/instance", get(get_local_instance))
        .route("/instance/follow", post(follow_instance))
}

#[derive(Deserialize, Serialize)]
pub struct CreateArticle {
    pub title: String,
    pub text: String,
}

#[debug_handler]
async fn create_article(
    data: Data<DatabaseHandle>,
    Form(create_article): Form<CreateArticle>,
) -> MyResult<Json<DbArticle>> {
    let local_instance_id = data.local_instance().ap_id;
    let ap_id = generate_object_id(local_instance_id.inner())?.into();
    let article = DbArticle {
        title: create_article.title,
        text: create_article.text,
        ap_id,
        instance: local_instance_id,
        local: true,
    };
    let mut articles = data.articles.lock().unwrap();
    articles.push(article.clone());
    Ok(Json(article))
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GetArticle {
    pub title: String,
}

#[debug_handler]
async fn get_article(
    Query(query): Query<GetArticle>,
    data: Data<DatabaseHandle>,
) -> MyResult<Json<DbArticle>> {
    let articles = data.articles.lock().unwrap();
    let article = articles
        .iter()
        .find(|a| a.title == query.title)
        .ok_or(anyhow!("not found"))?
        .clone();
    Ok(Json(article))
}

#[derive(Deserialize, Serialize)]
pub struct ResolveObject {
    pub id: Url,
}

#[debug_handler]
async fn resolve_object(
    Query(query): Query<ResolveObject>,
    data: Data<DatabaseHandle>,
) -> MyResult<Json<DbInstance>> {
    let instance: DbInstance = ObjectId::from(query.id).dereference(&data).await?;
    let mut instances = data.instances.lock().unwrap();
    instances.push(instance.clone());
    Ok(Json(instance))
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
