use crate::{
    activities::{reject::RejectEdit, update_local_article::UpdateLocalArticle},
    generate_activity_id,
    objects::{
        edit::{ApubEdit, EditWrapper},
        instance::InstanceWrapper,
    },
    send_activity,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::UpdateType,
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
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRemoteArticle {
    pub actor: ObjectId<InstanceWrapper>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    pub object: ApubEdit,
    #[serde(rename = "type")]
    pub kind: UpdateType,
    pub id: Url,
}

impl UpdateRemoteArticle {
    /// Sent by a follower instance
    pub async fn send(
        edit: EditWrapper,
        article_instance: Instance,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        let local_instance: InstanceWrapper = Instance::read_local(context)?.into();
        let id = generate_activity_id(context)?;
        let update = UpdateRemoteArticle {
            actor: local_instance.ap_id.clone().into(),
            to: vec![*article_instance.ap_id.0],
            object: edit.into_json(context).await?,
            kind: Default::default(),
            id,
        };
        send_activity(
            &local_instance,
            update,
            vec![Url::parse(&article_instance.inbox_url)?],
            context,
        )
        .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for UpdateRemoteArticle {
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
        let local_article = Article::read_from_ap_id(&self.object.object.clone().into(), context)?;
        let patch = Patch::from_str(&self.object.content)?;

        match apply(&local_article.text, &patch) {
            Ok(applied) => {
                let edit = EditWrapper::from_json(self.object.clone(), context).await?;
                let article = Article::update_text(edit.article_id, &applied, context)?;
                UpdateLocalArticle::send(
                    article.into(),
                    vec![self.actor.dereference(context).await?],
                    context,
                )
                .await?;
            }
            Err(_e) => {
                let user_instance = self.actor.dereference(context).await?;
                RejectEdit::send(self.object.clone(), user_instance, context).await?;
            }
        }

        Ok(())
    }
}
