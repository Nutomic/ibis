use crate::database::DatabaseHandle;

use crate::error::MyResult;
use crate::federation::activities::create_or_update_article::{
    CreateOrUpdateArticle, CreateOrUpdateType,
};
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
        .route(
            "/article",
            get(get_article).post(create_article).patch(edit_article),
        )
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
    {
        let mut articles = data.articles.lock().unwrap();
        articles.insert(article.ap_id.inner().clone(), article.clone());
    }

    CreateOrUpdateArticle::send_to_local_followers(
        article.clone(),
        CreateOrUpdateType::Create,
        &data,
    )
    .await?;

    Ok(Json(article))
}

#[derive(Deserialize, Serialize)]
pub struct EditArticle {
    pub ap_id: ObjectId<DbArticle>,
    pub new_text: String,
}

#[debug_handler]
async fn edit_article(
    data: Data<DatabaseHandle>,
    Form(edit_article): Form<EditArticle>,
) -> MyResult<Json<DbArticle>> {
    let article = {
        let mut lock = data.articles.lock().unwrap();
        let article = lock.get_mut(edit_article.ap_id.inner()).unwrap();
        article.text = edit_article.new_text;
        article.clone()
    };

    CreateOrUpdateArticle::send_to_local_followers(
        article.clone(),
        CreateOrUpdateType::Update,
        &data,
    )
    .await?;

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
        .find(|a| a.1.title == query.title)
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
async fn resolve_object(
    Query(query): Query<ResolveObject>,
    data: Data<DatabaseHandle>,
) -> MyResult<Json<DbInstance>> {
    let instance: DbInstance = ObjectId::from(query.id).dereference(&data).await?;
    let mut instances = data.instances.lock().unwrap();
    instances.insert(instance.ap_id.inner().clone(), instance.clone());
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
