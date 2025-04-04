use crate::objects::instance::{ApubInstance, InstanceWrapper};
use activitypub_federation::{
    config::Data,
    fetch::collection_id::CollectionId,
    kinds::collection::CollectionType,
    protocol::verification::verify_domains_match,
    traits::{Collection, Object},
};
use futures::future::{self, join_all};
use ibis_database::{
    common::{instance::Instance, utils::http_protocol_str},
    error::{BackendError, BackendResult},
    impls::IbisContext,
};
use log::warn;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubInstanceCollection {
    pub r#type: CollectionType,
    pub id: Url,
    pub total_items: i32,
    pub items: Vec<ApubInstance>,
}

#[derive(Clone, Debug)]
pub struct InstanceCollection(());

pub fn linked_instances_url(domain: &str) -> BackendResult<CollectionId<InstanceCollection>> {
    Ok(CollectionId::parse(&format!(
        "{}://{domain}/linked_instances",
        http_protocol_str()
    ))?)
}

#[async_trait::async_trait]
impl Collection for InstanceCollection {
    type Owner = ();
    type DataType = IbisContext;
    type Kind = ApubInstanceCollection;
    type Error = BackendError;

    async fn read_local(
        _owner: &Self::Owner,
        context: &Data<Self::DataType>,
    ) -> Result<Self::Kind, Self::Error> {
        let instances = Instance::list(context)?;
        let instances = future::try_join_all(
            instances
                .into_iter()
                .filter(|i| !i.local)
                .map(InstanceWrapper)
                .map(|i| i.into_json(context))
                .collect::<Vec<_>>(),
        )
        .await?;
        let collection = ApubInstanceCollection {
            r#type: Default::default(),
            id: linked_instances_url(&context.conf.federation.domain)?.into(),
            total_items: instances.len() as i32,
            items: instances,
        };
        Ok(collection)
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _context: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(&json.id, expected_domain)?;
        Ok(())
    }

    async fn from_json(
        apub: Self::Kind,
        _owner: &Self::Owner,
        context: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        let instances = apub
            .items
            .into_iter()
            .filter(|i| !i.id.is_local(context))
            .map(|instance| async {
                let id = instance.id.clone();
                let res = InstanceWrapper::from_json(instance, context).await;
                if let Err(e) = &res {
                    warn!("Failed to synchronize article {id}: {e}");
                }
                res
            });
        join_all(instances).await;

        Ok(InstanceCollection(()))
    }
}
