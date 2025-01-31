use super::empty_to_none;
use crate::{
    backend::{
        database::{instance::DbInstanceUpdateForm, IbisContext},
        federation::activities::follow::Follow,
        utils::error::BackendResult,
    },
    common::{
        instance::{
            DbInstance,
            FollowInstanceParams,
            GetInstanceParams,
            InstanceView,
            UpdateInstanceParams,
        },
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
    context: Data<IbisContext>,
    Form(params): Form<GetInstanceParams>,
) -> BackendResult<Json<InstanceView>> {
    let local_instance = DbInstance::read_view(params.id, &context)?;
    Ok(Json(local_instance))
}

pub(in crate::backend::api) async fn update_instance(
    context: Data<IbisContext>,
    Form(mut params): Form<UpdateInstanceParams>,
) -> BackendResult<Json<DbInstance>> {
    empty_to_none(&mut params.name);
    empty_to_none(&mut params.topic);
    let form = DbInstanceUpdateForm {
        name: params.name,
        topic: params.topic,
    };
    Ok(Json(DbInstance::update(form, &context)?))
}

/// Make the local instance follow a given remote instance, to receive activities about new and
/// updated articles.
#[debug_handler]
pub(in crate::backend::api) async fn follow_instance(
    Extension(user): Extension<LocalUserView>,
    context: Data<IbisContext>,
    Form(params): Form<FollowInstanceParams>,
) -> BackendResult<Json<SuccessResponse>> {
    let target = DbInstance::read(params.id, &context)?;
    let pending = !target.local;
    DbInstance::follow(&user.person, &target, pending, &context)?;
    let instance = DbInstance::read(params.id, &context)?;
    Follow::send(user.person, &instance, &context).await?;
    Ok(Json(SuccessResponse::default()))
}

/// Fetch a remote instance actor. This automatically synchronizes the remote articles collection to
/// the local instance, and allows for interactions such as following.
#[debug_handler]
pub(super) async fn resolve_instance(
    Query(params): Query<ResolveObjectParams>,
    context: Data<IbisContext>,
) -> BackendResult<Json<DbInstance>> {
    let instance: DbInstance = ObjectId::from(params.id).dereference(&context).await?;
    Ok(Json(instance))
}

#[debug_handler]
pub(in crate::backend::api) async fn list_instances(
    context: Data<IbisContext>,
) -> BackendResult<Json<Vec<DbInstance>>> {
    let instances = DbInstance::list(false, &context)?;
    Ok(Json(instances))
}
