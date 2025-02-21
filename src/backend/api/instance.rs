use super::{empty_to_none, UserExt};
use crate::{
    backend::{
        database::{
            instance::{DbInstanceUpdateForm, InstanceViewQuery},
            IbisContext,
        },
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
        utils::http_protocol_str,
        ResolveObjectParams,
        SuccessResponse,
    },
};
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use anyhow::anyhow;
use axum::{extract::Query, Form, Json};
use axum_macros::debug_handler;
use moka::sync::Cache;
use std::{sync::LazyLock, time::Duration};

/// Retrieve details about an instance. If no id is provided, return local instance.
#[debug_handler]
pub(in crate::backend::api) async fn get_instance(
    context: Data<IbisContext>,
    Form(params): Form<GetInstanceParams>,
) -> BackendResult<Json<InstanceView2>> {
    use InstanceViewQuery::*;
    let local_instance = match (params.id, params.hostname) {
        (Some(id), None) => Instance::read_view(Id(id), &context)?,
        (None, Some(hostname)) => {
            let url =
                ObjectId::<Instance>::parse(&format!("{}://{hostname}", http_protocol_str()))?;
            if let Ok(i) = Instance::read_view(ApId(&url), &context) {
                i
            } else {
                let id = url.dereference(&context).await?.id;
                Instance::read_view(Id(id), &context)?
            }
        }
        (None, None) => Instance::read_view(Local, &context)?,
        _ => return Err(anyhow!("invalid params").into()),
    };

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
    let instances = if cfg!(debug_assertions) {
        Instance::list_views(&context)?
    } else {
        // Cache result of the db read in prod because it uses a lot of queries and rarely changes
        static CACHE: LazyLock<Cache<(), Vec<InstanceView>>> = LazyLock::new(|| {
            Cache::builder()
                .max_capacity(1)
                .time_to_live(Duration::from_secs(60 * 60))
                .build()
        });
        CACHE
            .try_get_with((), || Instance::list_views(&context))
            .map_err(|e| anyhow!(e))?
    };
    Ok(Json(instances))
}
