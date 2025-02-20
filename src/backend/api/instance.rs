use super::{empty_to_none, UserExt};
use crate::{
    backend::{
        database::{instance::DbInstanceUpdateForm, IbisContext},
        federation::activities::follow::Follow,
        utils::error::BackendResult,
    },
    common::{
        instance::{
            FollowInstanceParams,
            GetInstanceParams,
            Instance,
            InstanceView,
            InstanceView2,
            UpdateInstanceParams,
        },
        ResolveObjectParams,
        SuccessResponse,
    },
};
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use axum::{extract::Query, Form, Json};
use axum_macros::debug_handler;

/// Retrieve details about an instance. If no id is provided, return local instance.
#[debug_handler]
pub(in crate::backend::api) async fn get_instance(
    context: Data<IbisContext>,
    Form(params): Form<GetInstanceParams>,
) -> BackendResult<Json<InstanceView2>> {
    let local_instance = Instance::read_view(params.id, &context)?;
    Ok(Json(local_instance))
}

pub(in crate::backend::api) async fn update_instance(
    context: Data<IbisContext>,
    Form(mut params): Form<UpdateInstanceParams>,
) -> BackendResult<Json<Instance>> {
    empty_to_none(&mut params.name);
    empty_to_none(&mut params.topic);
    let form = DbInstanceUpdateForm {
        name: params.name,
        topic: params.topic,
    };
    Ok(Json(Instance::update(form, &context)?))
}

/// Make the local instance follow a given remote instance, to receive activities about new and
/// updated articles.
#[debug_handler]
pub(in crate::backend::api) async fn follow_instance(
    user: UserExt,
    context: Data<IbisContext>,
    Form(params): Form<FollowInstanceParams>,
) -> BackendResult<Json<SuccessResponse>> {
    let target = Instance::read(params.id, &context)?;
    let pending = !target.local;
    Instance::follow(&user.person, &target, pending, &context)?;
    let instance = Instance::read(params.id, &context)?;
    Follow::send(user.inner().person, &instance, &context).await?;
    Ok(Json(SuccessResponse::default()))
}

/// Fetch a remote instance actor. This automatically synchronizes the remote articles collection to
/// the local instance, and allows for interactions such as following.
#[debug_handler]
pub(super) async fn resolve_instance(
    Query(params): Query<ResolveObjectParams>,
    context: Data<IbisContext>,
) -> BackendResult<Json<Instance>> {
    let instance: Instance = ObjectId::from(params.id).dereference(&context).await?;
    Ok(Json(instance))
}

#[debug_handler]
pub(in crate::backend::api) async fn list_instances(
    context: Data<IbisContext>,
) -> BackendResult<Json<Vec<Instance>>> {
    let instances = Instance::list(&context)?;
    Ok(Json(instances))
}

#[debug_handler]
pub(in crate::backend::api) async fn list_instance_views(
    context: Data<IbisContext>,
) -> BackendResult<Json<Vec<InstanceView>>> {
    let instances = Instance::list_views(&context)?;
    Ok(Json(instances))
}
