use super::update_article::UpdateArticle;
use crate::{
    AnnounceActivity,
    activities::reject::RejectEdit,
    generate_activity_id,
    objects::{
        edit::{ApubEdit, EditWrapper},
        instance::InstanceWrapper,
        user::PersonWrapper,
    },
    routes::AnnouncableActivities,
    send_activity,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::public,
    protocol::helpers::deserialize_one_or_many,
    traits::{ActivityHandler, Object},
};
use diffy::{Patch, apply};
use ibis_database::{
    common::{
        article::{Article, can_edit_article},
        instance::Instance,
    },
    error::{BackendError, BackendResult},
    impls::IbisContext,
};
use log::warn;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EditArticle {
    pub actor: ObjectId<PersonWrapper>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    pub object: ApubEdit,
    #[serde(rename = "type")]
    pub kind: EditType,
    pub id: Url,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub enum EditType {
    #[default]
    Edit,
}

impl EditArticle {
    pub async fn new(
        edit: EditWrapper,
        from: &PersonWrapper,
        to_instance: &InstanceWrapper,
        context: &Data<IbisContext>,
    ) -> BackendResult<Self> {
        let id = generate_activity_id(context)?;
        Ok(EditArticle {
            actor: from.ap_id.clone().into(),
            to: vec![to_instance.ap_id.clone().into(), public()],
            object: edit.into_json(context).await?,
            kind: Default::default(),
            id,
        })
    }
    pub async fn send(
        self,
        from: &PersonWrapper,
        to_instance: &InstanceWrapper,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        send_activity(
            from,
            self,
            vec![Url::parse(&to_instance.inbox_url)?],
            context,
        )
        .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for EditArticle {
    type DataType = IbisContext;
    type Error = BackendError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let article = Article::read_from_ap_id(&self.object.object.clone().into(), context)?;
        can_edit_article(&article, false)?;
        Ok(())
    }

    /// Received on article origin instance
    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let article = Article::read_from_ap_id(&self.object.object.clone().into(), context)?;
        let patch = Patch::from_str(&self.object.content)?;
        let actor = self.actor.dereference(context).await?;

        match apply(&article.text, &patch) {
            Ok(applied) => {
                let edit = EditWrapper::from_json(self.object.clone(), context).await?;
                let article = Article::update_text(edit.article_id, &applied, context)?;
                if article.local {
                    AnnounceActivity::send(AnnouncableActivities::EditArticle(self), context)
                        .await?;
                    let local_instance: InstanceWrapper = Instance::read_local(context)?.into();
                    UpdateArticle::send(actor, article.into(), &local_instance, context).await?;
                }
            }
            Err(_e) if article.local => {
                RejectEdit::send(self.object.clone(), actor, context).await?;
            }
            Err(e) => {
                warn!("Failed to apply federated edit: {e}")
            }
        }

        Ok(())
    }
}
