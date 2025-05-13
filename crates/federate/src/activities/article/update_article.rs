use crate::{
    activities::announce::AnnounceActivity,
    generate_activity_id,
    objects::{
        article::{ApubArticle, ArticleWrapper},
        user::PersonWrapper,
    },
    routes::AnnouncableActivities,
    send_ibis_activity,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{activity::UpdateType, public},
    protocol::helpers::deserialize_one_or_many,
    traits::{ActivityHandler, Object},
};
use ibis_database::{
    common::user::Person,
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
    pub(crate) async fn new(
        article: ArticleWrapper,
        context: &Data<IbisContext>,
    ) -> BackendResult<Self> {
        let object = article.clone().into_json(context).await?;
        let actor = Person::wikibot(context)?;
        let id = generate_activity_id(context)?;
        Ok(UpdateArticle {
            actor: actor.ap_id.clone().into(),
            to: vec![public()],
            cc: vec![],
            object,
            kind: Default::default(),
            id,
        })
    }

    pub async fn send(article: ArticleWrapper, context: &Data<IbisContext>) -> BackendResult<()> {
        let update = Self::new(article, context).await?;
        Self::send_update(update, context).await?;
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
            Self::send_update(self, context).await?;
        }
        Ok(())
    }
}

impl UpdateArticle {
    async fn send_update(self, context: &Data<IbisContext>) -> BackendResult<()> {
        let actor: PersonWrapper = Person::wikibot(context)?.into();
        let inboxes: Vec<_> = Person::read_followers(actor.id, context)?
            .iter()
            .map(|f| f.inbox_url())
            .collect();
        send_ibis_activity(&actor, self.clone(), inboxes, context).await?;

        let announce = AnnouncableActivities::UpdateArticle(self);
        AnnounceActivity::send(announce, context).await?;
        Ok(())
    }
}
