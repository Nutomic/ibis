use crate::database::DatabaseHandle;
use crate::error::MyResult;
use crate::federation::objects::article::DbArticle;
use crate::federation::objects::instance::DbInstance;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use axum::extract::{Path, Query};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};

use url::Url;

pub fn api_routes() -> Router {
    Router::new()
        .route("/article/:title", get(get_article))
        .route("/resolve_object", get(resolve_object))
        .route("/instance", get(get_local_instance))
        .route("/instance/follow", post(follow_instance))
}

#[debug_handler]
async fn get_article(
    Path(title): Path<String>,
    data: Data<DatabaseHandle>,
) -> MyResult<Json<DbArticle>> {
    let instance = data.local_instance();
    let article = DbArticle::new(title, "dummy".to_string(), instance.ap_id)?;
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
