use crate::backend::database::instance::{DbInstance, InstanceView};
use crate::backend::database::MyDataHandle;
use crate::backend::error::MyResult;
use crate::backend::federation::activities::follow::Follow;
use crate::common::LocalUserView;
use activitypub_federation::config::Data;
use axum::Extension;
use axum::{Form, Json};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};

/// Retrieve the local instance info.
#[debug_handler]
pub(in crate::backend::api) async fn get_local_instance(
    data: Data<MyDataHandle>,
) -> MyResult<Json<InstanceView>> {
    let local_instance = DbInstance::read_local_view(&data.db_connection)?;
    Ok(Json(local_instance))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FollowInstance {
    pub id: i32,
}

/// Make the local instance follow a given remote instance, to receive activities about new and
/// updated articles.
#[debug_handler]
pub(in crate::backend::api) async fn follow_instance(
    Extension(user): Extension<LocalUserView>,
    data: Data<MyDataHandle>,
    Form(query): Form<FollowInstance>,
) -> MyResult<()> {
    let target = DbInstance::read(query.id, &data.db_connection)?;
    let pending = !target.local;
    DbInstance::follow(&user.person, &target, pending, &data)?;
    let instance = DbInstance::read(query.id, &data.db_connection)?;
    Follow::send(user.person, instance, &data).await?;
    Ok(())
}
