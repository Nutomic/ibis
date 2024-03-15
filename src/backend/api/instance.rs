use crate::{
    backend::{database::IbisData, error::MyResult, federation::activities::follow::Follow},
    common::{DbInstance, FollowInstance, GetInstance, InstanceView, LocalUserView, ResolveObject},
};
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use axum::{extract::Query, Extension, Form, Json};
use axum_macros::debug_handler;

/// Retrieve details about an instance. If no id is provided, return local instance.
#[debug_handler]
pub(in crate::backend::api) async fn get_instance(
    data: Data<IbisData>,
    Form(query): Form<GetInstance>,
) -> MyResult<Json<InstanceView>> {
    let local_instance = DbInstance::read_view(query.id, &data)?;
    Ok(Json(local_instance))
}

/// Make the local instance follow a given remote instance, to receive activities about new and
/// updated articles.
#[debug_handler]
pub(in crate::backend::api) async fn follow_instance(
    Extension(user): Extension<LocalUserView>,
    data: Data<IbisData>,
    Form(query): Form<FollowInstance>,
) -> MyResult<()> {
    let target = DbInstance::read(query.id, &data)?;
    let pending = !target.local;
    DbInstance::follow(&user.person, &target, pending, &data)?;
    let instance = DbInstance::read(query.id, &data)?;
    Follow::send(user.person, &instance, &data).await?;
    Ok(())
}

/// Fetch a remote instance actor. This automatically synchronizes the remote articles collection to
/// the local instance, and allows for interactions such as following.
#[debug_handler]
pub(super) async fn resolve_instance(
    Query(query): Query<ResolveObject>,
    data: Data<IbisData>,
) -> MyResult<Json<DbInstance>> {
    let instance: DbInstance = ObjectId::from(query.id).dereference(&data).await?;
    Ok(Json(instance))
}
