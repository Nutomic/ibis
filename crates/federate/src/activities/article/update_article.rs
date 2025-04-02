use crate::{
    generate_activity_id,
    objects::{
        article::{ApubArticle, ArticleWrapper},
        instance::InstanceWrapper,
        user::PersonWrapper,
    },
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{activity::UpdateType, public},
    protocol::helpers::deserialize_one_or_many,
    traits::{ActivityHandler, Object},
};
use ibis_database::{
    common::{instance::Instance, user::Person},
    error::{BackendError, BackendResult},
    impls::IbisContext,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArticle {
    pub actor: ObjectId<PersonWrapper>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub cc: Vec<Url>,
    pub object: ApubArticle,
    #[serde(rename = "type")]
    pub kind: UpdateType,
    pub id: Url,
}

impl UpdateArticle {
    pub async fn send(
        article: ArticleWrapper,
        local_instance: &InstanceWrapper,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        let object = article.clone().into_json(context).await?;
        let actor = Person::wikibot(context)?;
        let id = generate_activity_id(context)?;
        let create = UpdateArticle {
            actor: actor.ap_id.clone().into(),
            to: vec![public()],
            cc: vec![],
            object,
            kind: Default::default(),
            id,
        };
        local_instance
            .send_to_followers(create, vec![], context)
            .await?;
        Ok(())
    }
}
#[async_trait::async_trait]
impl ActivityHandler for UpdateArticle {
    type DataType = IbisContext;
    type Error = BackendError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let article = ArticleWrapper::from_json(self.object.clone(), context).await?;
        if article.local {
            let local_instance: InstanceWrapper = Instance::read_local(context)?.into();
            local_instance
                .send_to_followers(self, vec![], context)
                .await?;
        }
        Ok(())
    }
}
