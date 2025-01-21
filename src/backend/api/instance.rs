use crate::{
    backend::{database::IbisData, federation::activities::follow::Follow, utils::error::MyResult},
    common::{
        instance::{DbInstance, FollowInstanceParams, GetInstanceParams, InstanceView},
        user::LocalUserView,
        ResolveObjectParams,
        SuccessResponse,
    },
};
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use axum::{extract::Query, Extension, Form, Json};
use axum_macros::debug_handler;

/// Retrieve details about an instance. If no id is provided, return local instance.
#[debug_handler]
pub(in crate::backend::api) async fn get_instance(
    data: Data<IbisData>,
    Form(params): Form<GetInstanceParams>,
) -> MyResult<Json<InstanceView>> {
    let local_instance = DbInstance::read_view(params.id, &data)?;
    Ok(Json(local_instance))
}

/// Make the local instance follow a given remote instance, to receive activities about new and
/// updated articles.
#[debug_handler]
pub(in crate::backend::api) async fn follow_instance(
    Extension(user): Extension<LocalUserView>,
    data: Data<IbisData>,
    Form(params): Form<FollowInstanceParams>,
) -> MyResult<Json<SuccessResponse>> {
    let target = DbInstance::read(params.id, &data)?;
    let pending = !target.local;
    DbInstance::follow(&user.person, &target, pending, &data)?;
    let instance = DbInstance::read(params.id, &data)?;
    Follow::send(user.person, &instance, &data).await?;
    Ok(Json(SuccessResponse::default()))
}

/// Fetch a remote instance actor. This automatically synchronizes the remote articles collection to
/// the local instance, and allows for interactions such as following.
#[debug_handler]
pub(super) async fn resolve_instance(
    Query(params): Query<ResolveObjectParams>,
    data: Data<IbisData>,
) -> MyResult<Json<DbInstance>> {
    let instance: DbInstance = ObjectId::from(params.id).dereference(&data).await?;
    Ok(Json(instance))
}

#[debug_handler]
pub(in crate::backend::api) async fn list_remote_instances(
    data: Data<IbisData>,
) -> MyResult<Json<Vec<DbInstance>>> {
    let instances = DbInstance::read_remote(&data)?;
    Ok(Json(instances))
}
