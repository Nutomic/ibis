use crate::backend::database::MyDataHandle;
use crate::backend::error::MyResult;
use crate::backend::federation::activities::follow::Follow;
use crate::common::{DbInstance, InstanceView, ResolveObject};
use crate::common::{FollowInstance, LocalUserView};
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use axum::extract::Query;
use axum::Extension;
use axum::{Form, Json};
use axum_macros::debug_handler;

/// Retrieve the local instance info.
#[debug_handler]
pub(in crate::backend::api) async fn get_local_instance(
    data: Data<MyDataHandle>,
) -> MyResult<Json<InstanceView>> {
    let local_instance = DbInstance::read_local_view(&data.db_connection)?;
    Ok(Json(local_instance))
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

/// Fetch a remote instance actor. This automatically synchronizes the remote articles collection to
/// the local instance, and allows for interactions such as following.
#[debug_handler]
pub(super) async fn resolve_instance(
    Query(query): Query<ResolveObject>,
    data: Data<MyDataHandle>,
) -> MyResult<Json<DbInstance>> {
    // TODO: workaround because axum makes it hard to have multiple routes on /
    let instance: DbInstance = ObjectId::from(query.id).dereference(&data).await?;
    Ok(Json(instance))
}
