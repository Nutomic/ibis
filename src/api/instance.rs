use crate::database::instance::{DbInstance, InstanceView};
use crate::database::MyDataHandle;
use crate::error::MyResult;
use crate::federation::activities::follow::Follow;
use activitypub_federation::config::Data;
use axum::{Form, Json};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};

/// Retrieve the local instance info.
#[debug_handler]
pub(in crate::api) async fn get_local_instance(
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
pub(in crate::api) async fn follow_instance(
    data: Data<MyDataHandle>,
    Form(query): Form<FollowInstance>,
) -> MyResult<()> {
    let local_instance = DbInstance::read_local_instance(&data.db_connection)?;
    let target = DbInstance::read(query.id, &data.db_connection)?;
    let pending = !target.local;
    DbInstance::follow(local_instance.id, target.id, pending, &data)?;
    let instance = DbInstance::read(query.id, &data.db_connection)?;
    Follow::send(local_instance, instance, &data).await?;
    Ok(())
}
