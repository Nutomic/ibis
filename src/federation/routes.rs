use crate::error::Error;
use crate::federation::objects::person::{DbUser, Person, PersonAcceptedActivities};
use crate::instance::DatabaseHandle;
use activitypub_federation::axum::inbox::{receive_activity, ActivityData};
use activitypub_federation::axum::json::FederationJson;
use activitypub_federation::config::Data;
use activitypub_federation::protocol::context::WithContext;
use activitypub_federation::traits::Object;
use axum::extract::path::Path;
use axum::response::IntoResponse;
use axum_macros::debug_handler;

#[debug_handler]
pub async fn http_get_user(
    Path(name): Path<String>,
    data: Data<DatabaseHandle>,
) -> Result<FederationJson<WithContext<Person>>, Error> {
    let db_user = data.read_user(&name)?;
    let json_user = db_user.into_json(&data).await?;
    Ok(FederationJson(WithContext::new_default(json_user)))
}

#[debug_handler]
pub async fn http_post_user_inbox(
    data: Data<DatabaseHandle>,
    activity_data: ActivityData,
) -> impl IntoResponse {
    receive_activity::<WithContext<PersonAcceptedActivities>, DbUser, DatabaseHandle>(
        activity_data,
        &data,
    )
    .await
}
