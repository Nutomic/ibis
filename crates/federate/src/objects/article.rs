use super::{edits_collection::EditCollection, instance::InstanceWrapper};
use crate::validate::validate_article_title;
use activitypub_federation::{
    config::Data,
    fetch::{collection_id::CollectionId, object_id::ObjectId},
    kinds::{object::ArticleType, public},
    protocol::{
        helpers::deserialize_one_or_many,
        verification::{verify_domains_match, verify_is_remote_object},
    },
    traits::Object,
};
use ibis_database::{
    common::{
        article::{Article, EditVersion},
        instance::Instance,
    },
    error::BackendError,
    impls::{IbisContext, article::DbArticleForm},
};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApubArticle {
    #[serde(rename = "type")]
    pub kind: ArticleType,
    pub id: ObjectId<ArticleWrapper>,
    pub attributed_to: ObjectId<InstanceWrapper>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    pub edits: CollectionId<EditCollection>,
    latest_version: EditVersion,
    content: String,
    name: String,
    protected: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ArticleWrapper(pub Article);

impl Deref for ArticleWrapper {
    type Target = Article;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Article> for ArticleWrapper {
    fn from(value: Article) -> Self {
        ArticleWrapper(value)
    }
}

#[async_trait::async_trait]
impl Object for ArticleWrapper {
    type DataType = IbisContext;
    type Kind = ApubArticle;
    type Error = BackendError;

    async fn read_from_id(
        object_id: Url,
        context: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        let article = Article::read_from_ap_id(&object_id.into(), context).ok();
        Ok(article.map(Into::into))
    }

    async fn into_json(self, context: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let local_instance: InstanceWrapper = Instance::read_local(context)?.into();
        Ok(ApubArticle {
            kind: Default::default(),
            id: self.ap_id.clone().into(),
            attributed_to: local_instance.ap_id.clone().into(),
            to: vec![public(), local_instance.followers_url()?],
            edits: self.edits_id()?.into(),
            latest_version: self.latest_edit_version(context)?,
            content: self.text.clone(),
            name: self.title.clone(),
            protected: self.protected,
        })
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        context: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id.inner(), expected_domain)?;
        verify_is_remote_object(&json.id, context)?;
        Ok(())
    }

    async fn from_json(
        json: Self::Kind,
        context: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        let instance = json.attributed_to.dereference(context).await?;
        let mut form = DbArticleForm {
            title: json.name,
            text: json.content,
            ap_id: json.id.into(),
            local: false,
            instance_id: instance.id,
            protected: json.protected,
            approved: true,
        };
        form.title = validate_article_title(&form.title)?;
        let article = Article::create_or_update(form, context)?;

        json.edits.dereference(&article, context).await?;

        Ok(article.into())
    }
}
