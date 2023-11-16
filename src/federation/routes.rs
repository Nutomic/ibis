use crate::database::DatabaseHandle;
use crate::error::MyResult;
use crate::federation::activities::accept::Accept;
use crate::federation::activities::follow::Follow;
use crate::federation::objects::instance::{DbInstance, Instance};

use activitypub_federation::axum::inbox::{receive_activity, ActivityData};
use activitypub_federation::axum::json::FederationJson;
use activitypub_federation::config::Data;
use activitypub_federation::protocol::context::WithContext;
use activitypub_federation::traits::ActivityHandler;
use activitypub_federation::traits::Object;

use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use url::Url;

pub fn federation_routes() -> Router {
    Router::new()
        .route("/", get(http_get_instance))
        .route("/inbox", post(http_post_inbox))
}

#[debug_handler]
async fn http_get_instance(
    data: Data<DatabaseHandle>,
) -> MyResult<FederationJson<WithContext<Instance>>> {
    let db_instance = data.local_instance();
    let json_instance = db_instance.into_json(&data).await?;
    Ok(FederationJson(WithContext::new_default(json_instance)))
}

/// List of all activities which this actor can receive.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum InboxActivities {
    Follow(Follow),
    Accept(Accept),
}

#[debug_handler]
pub async fn http_post_inbox(
    data: Data<DatabaseHandle>,
    activity_data: ActivityData,
) -> impl IntoResponse {
    dbg!("receive activity");
    receive_activity::<WithContext<InboxActivities>, DbInstance, DatabaseHandle>(
        activity_data,
        &data,
    )
    .await
}
