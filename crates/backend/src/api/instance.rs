use super::{UserExt, empty_to_none};
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use anyhow::anyhow;
use axum::{Form, Json, extract::Query};
use axum_macros::debug_handler;
use ibis_api_client::instance::{FollowInstanceParams, GetInstanceParams, UpdateInstanceParams};
use ibis_database::{
    common::{
        ResolveObjectParams,
        SuccessResponse,
        instance::{Instance, InstanceView, InstanceWithArticles},
        utils::http_protocol_str,
    },
    error::BackendResult,
    impls::{
        IbisContext,
        instance::{DbInstanceUpdateForm, InstanceViewQuery},
    },
};
use ibis_federate::{
    activities::{follow::Follow, undo_follow::UndoFollow},
    objects::instance::InstanceWrapper,
};
use moka::sync::Cache;
use std::{sync::LazyLock, time::Duration};

/// Retrieve details about an instance. If no id is provided, return local instance.
#[debug_handler]
pub(crate) async fn get_instance(
    context: Data<IbisContext>,
    Form(params): Form<GetInstanceParams>,
) -> BackendResult<Json<InstanceView>> {
    use InstanceViewQuery::*;
    let instance = match (params.id, params.hostname) {
        (Some(id), None) => Instance::read_view(Id(id), &context)?,
        (None, Some(hostname)) => {
            if let Ok(i) = Instance::read_view(Hostname(&hostname), &context) {
                i
            } else {
                let id = ObjectId::<InstanceWrapper>::parse(&format!(
                    "{}://{hostname}",
                    http_protocol_str()
                ))?
                .dereference(&context)
                .await?
                .id;
                Instance::read_view(Id(id), &context)?
            }
        }
        _ => return Err(anyhow!("invalid params").into()),
    };

    Ok(Json(instance))
}

pub(crate) async fn update_instance(
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
pub(crate) async fn follow_instance(
    user: UserExt,
    context: Data<IbisContext>,
    Form(params): Form<FollowInstanceParams>,
) -> BackendResult<Json<SuccessResponse>> {
    let instance = Instance::read(params.id, &context)?;
    let person = user.person.clone();
    let actor = user.inner().person.into();
    if params.follow {
        let pending = !instance.local;
        Instance::follow(&person, &instance, pending, &context)?;
        Follow::send(&actor, &instance.into(), &context).await?;
    } else {
        Instance::unfollow(&person, &instance, &context)?;
        UndoFollow::send(&actor, &instance.into(), &context).await?;
    }
    Ok(Json(SuccessResponse::default()))
}

/// Fetch a remote instance actor. This automatically synchronizes the remote articles collection to
/// the local instance, and allows for interactions such as following.
#[debug_handler]
pub(super) async fn resolve_instance(
    Query(params): Query<ResolveObjectParams>,
    context: Data<IbisContext>,
) -> BackendResult<Json<Instance>> {
    let instance: InstanceWrapper = ObjectId::from(params.id).dereference(&context).await?;
    Ok(Json(instance.0))
}

#[debug_handler]
pub(crate) async fn list_instance_views(
    context: Data<IbisContext>,
) -> BackendResult<Json<Vec<InstanceWithArticles>>> {
    let instances = if cfg!(debug_assertions) {
        Instance::list_with_articles(&context)?
    } else {
        // Cache result of the db read in prod because it uses a lot of queries and rarely changes
        static CACHE: LazyLock<Cache<(), Vec<InstanceWithArticles>>> = LazyLock::new(|| {
            Cache::builder()
                .max_capacity(1)
                .time_to_live(Duration::from_secs(60 * 60))
                .build()
        });
        CACHE
            .try_get_with((), || Instance::list_with_articles(&context))
            .map_err(|e| anyhow!(e))?
    };
    Ok(Json(instances))
}
