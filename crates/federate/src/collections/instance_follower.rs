use activitypub_federation::{
    config::Data,
    kinds::collection::CollectionType,
    protocol::verification::verify_domains_match,
    traits::Collection,
};
use ibis_database::{common::instance::Instance, error::BackendError, impls::IbisContext};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GroupFollowers {
    pub(crate) id: Url,
    pub(crate) r#type: CollectionType,
    pub(crate) total_items: i32,
    pub(crate) items: Vec<()>,
}

#[derive(Clone, Debug)]
pub(crate) struct InstanceFollower(());

#[async_trait::async_trait]
impl Collection for InstanceFollower {
    type Owner = ();
    type DataType = IbisContext;
    type Kind = GroupFollowers;
    type Error = BackendError;

    async fn read_local(
        _community: &Self::Owner,
        context: &Data<Self::DataType>,
    ) -> Result<Self::Kind, Self::Error> {
        let instance = Instance::read_local(context)?;
        let followers = Instance::read_local_followers_count(context)?;

        Ok(GroupFollowers {
            id: Url::parse(&format!("{}followers", instance.ap_id))?,
            r#type: CollectionType::Collection,
            total_items: followers as i32,
            items: vec![],
        })
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(expected_domain, &json.id)?;
        Ok(())
    }

    async fn from_json(
        _json: Self::Kind,
        _community: &Self::Owner,
        _context: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        Ok(InstanceFollower(()))
    }
}
