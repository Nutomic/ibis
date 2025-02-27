use crate::{
    generate_activity_id,
    objects::{article::ArticleWrapper, instance::InstanceWrapper, user::PersonWrapper},
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{activity::DeleteType, public},
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
pub struct RemoveArticle {
    pub(crate) actor: ObjectId<PersonWrapper>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: ObjectId<ArticleWrapper>,
    #[serde(rename = "type")]
    pub(crate) kind: DeleteType,
    pub(crate) id: Url,

    pub(crate) summary: String,
}

impl RemoveArticle {
    pub(super) fn new(
        actor: ObjectId<PersonWrapper>,
        article: ArticleWrapper,
        context: &Data<IbisContext>,
    ) -> BackendResult<Self> {
        let object = article.ap_id.clone().into();
        let id = generate_activity_id(context)?;
        Ok(RemoveArticle {
            actor,
            to: vec![public()],
            object,
            kind: Default::default(),
            id,
            summary: "Removed by admin".to_string(),
        })
    }
    pub async fn send_to_followers(
        actor: ObjectId<PersonWrapper>,
        article: ArticleWrapper,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        let local_instance: InstanceWrapper = Instance::read_local(context)?.into();
        let remove = Self::new(actor, article, context)?;
        dbg!("send remove article");
        local_instance
            .send_to_followers(remove, vec![], context)
            .await?;
        Ok(())
    }
}
#[async_trait::async_trait]
impl ActivityHandler for RemoveArticle {
    type DataType = IbisContext;
    type Error = BackendError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        dbg!("verify remove article");
        verify_domains_match(self.actor.inner(), self.object.inner())?;
        Ok(())
    }

    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        dbg!("receive remove article");
        let article = Article::read_from_ap_id(&self.object.into_inner().into(), context);
        if let Ok(article) = article {
            Article::update_removed(article.id, true, context)?;
        }
        Ok(())
    }
}
