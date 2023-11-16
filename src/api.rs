use crate::error::MyResult;
use crate::federation::objects::article::DbArticle;
use crate::federation::objects::instance::DbInstance;
use axum::extract::Path;
use axum::routing::get;
use axum::{Json, Router};
use axum_macros::debug_handler;

pub fn api_routes() -> Router {
    Router::new().route("/article/:title", get(api_get_article))
}

#[debug_handler]
async fn api_get_article(Path(title): Path<String>) -> MyResult<Json<DbArticle>> {
    let instance = DbInstance::new("localhost")?;
    let article = DbArticle::new(title, "dummy".to_string(), instance.ap_id)?;
    Ok(Json(article))
}
