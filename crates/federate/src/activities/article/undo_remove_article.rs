use super::remove_article::RemoveArticle;
use crate::{
    generate_activity_id,
    objects::{article::ArticleWrapper, instance::InstanceWrapper, user::PersonWrapper},
    routes::AnnouncableActivities,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{activity::UndoType, public},
    protocol::{helpers::deserialize_one_or_many, verification::verify_domains_match},
    traits::ActivityHandler,
};
use ibis_database::{
    common::{article::Article, instance::Instance},
    error::{BackendError, BackendResult},
    impls::IbisContext,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoRemoveArticle {
    pub(crate) actor: ObjectId<PersonWrapper>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: RemoveArticle,
    #[serde(rename = "type")]
    pub(crate) kind: UndoType,
    pub(crate) id: Url,
}

impl UndoRemoveArticle {
    pub async fn send(
        actor: ObjectId<PersonWrapper>,
        article: ArticleWrapper,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        let object = RemoveArticle::new(actor.clone(), article, context)?;
        let id = generate_activity_id(context)?;
        let undo = UndoRemoveArticle {
            actor,
            to: vec![public()],
            object,
            kind: Default::default(),
            id,
        };
        let announce = AnnouncableActivities::UndoRemoveArticle(undo);

        let local_instance: InstanceWrapper = Instance::read_local(context)?.into();
        local_instance
            .send_to_followers(announce, vec![], context)
            .await?;
        Ok(())
    }
}
#[async_trait::async_trait]
impl ActivityHandler for UndoRemoveArticle {
    type DataType = IbisContext;
    type Error = BackendError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(self.actor.inner(), self.object.object.inner())?;
        Ok(())
    }

    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let article = Article::read_from_ap_id(&self.object.object.into_inner().into(), context);
        if let Ok(article) = article {
            Article::update_removed(article.id, false, context)?;
        }
        Ok(())
    }
}
