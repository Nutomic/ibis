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
    send_ibis_activity,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::public,
    protocol::helpers::deserialize_one_or_many,
    traits::{ActivityHandler, Object},
};
use anyhow::anyhow;
use chrono::Utc;
use diffy::{Patch, apply};
use ibis_database::{
    common::{
        article::{Article, Edit, can_edit_article},
        instance::Instance,
    },
    error::{BackendError, BackendResult},
    impls::{IbisContext, article::DbArticleForm},
};
use log::warn;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrEditArticle {
    pub actor: ObjectId<PersonWrapper>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    pub object: ApubEdit,
    #[serde(rename = "type")]
    pub kind: CreateOrEditType,
    pub id: Url,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum CreateOrEditType {
    Create,
    Edit,
}

impl CreateOrEditArticle {
    pub(crate) async fn new(
        edit: EditWrapper,
        from: &PersonWrapper,
        to_instance: &InstanceWrapper,
        is_create: bool,
        context: &Data<IbisContext>,
    ) -> BackendResult<Self> {
        let id = generate_activity_id(context)?;
        let kind = match is_create {
            true => CreateOrEditType::Create,
            false => CreateOrEditType::Edit,
        };
        Ok(CreateOrEditArticle {
            actor: from.ap_id.clone().into(),
            to: vec![to_instance.ap_id.clone().into(), public()],
            object: edit.into_json(context).await?,
            kind,
            id,
        })
    }
    pub(crate) async fn send(
        self,
        from: &PersonWrapper,
        to_instance: &InstanceWrapper,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        send_ibis_activity(
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
impl ActivityHandler for CreateOrEditArticle {
    type DataType = IbisContext;
    type Error = BackendError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let article = Article::read_from_ap_id(&self.object.object.clone().into(), context);
        if self.kind == CreateOrEditType::Create {
            let local_instance = Instance::read_local(context)?;
            if article.is_ok() && self.id.domain() != local_instance.ap_id.0.domain() {
                return Err(anyhow!("Article already exists").into());
            }
        } else {
            can_edit_article(&article?, false)?;
        }
        Ok(())
    }

    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let article = if self.kind == CreateOrEditType::Create {
            // remote user is creating new article on our instance
            let local_instance = Instance::read_local(context)?;
            if !self.to.contains(&local_instance.ap_id.clone().into()) {
                // not meant for us, ignore (not sure why this is being sent)
                return Ok(());
            }
            if self.object.object.inner().domain() != local_instance.ap_id.0.domain() {
                return Err(anyhow!("Invalid article ap id").into());
            }
            // last path segment is the title
            let title = self.object.object.to_string();
            let title = title
                .rsplit_once('/')
                .ok_or(anyhow!("Missing article title"))?
                .1
                .replace("_", " ");
            let form = DbArticleForm {
                title,
                text: String::new(),
                ap_id: self.object.object.clone().into(),
                instance_id: local_instance.id,
                local: true,
                protected: false,
                updated: Utc::now(),
                pending: false,
            };
            let creator = self.actor.dereference(context).await?;
            Article::create_or_update(form, creator.id, context).await?
        } else {
            Article::read_from_ap_id(&self.object.object.clone().into(), context)?
        };

        let edits = Edit::list_for_article(article.id, context)?;
        let edit_known = edits.into_iter().any(|e| e.hash == self.object.version);
        if edit_known {
            return Ok(());
        }

        let patch = Patch::from_str(&self.object.content)?;
        let actor = self.actor.dereference(context).await?;

        match apply(&article.text, &patch) {
            Ok(applied) => {
                let edit = EditWrapper::from_json(self.object.clone(), context).await?;
                let article = Article::update_text(edit.article_id, &applied, context)?;
                if article.local {
                    AnnounceActivity::send(AnnouncableActivities::EditArticle(self), context)
                        .await?;
                    UpdateArticle::send(article.into(), context).await?;
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
