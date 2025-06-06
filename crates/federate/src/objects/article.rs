use super::{Source, read_from_string_or_source, user::PersonWrapper};
use crate::{
    collections::edits_collection::EditCollection,
    objects::instance::InstanceWrapper,
    validate::validate_article_title,
};
use activitypub_federation::{
    config::Data,
    fetch::{collection_id::CollectionId, object_id::ObjectId},
    kinds::{object::ArticleType, public},
    protocol::{
        helpers::{deserialize_one_or_many, deserialize_skip_error},
        values::MediaTypeMarkdownOrHtml,
        verification::{verify_domains_match, verify_is_remote_object},
    },
    traits::Object,
};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use ibis_database::{
    common::{
        article::{Article, EditVersion},
        instance::Instance,
        user::Person,
    },
    error::BackendError,
    impls::{IbisContext, article::DbArticleForm},
};
use ibis_markdown::render_article_markdown;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{cmp::Reverse, ops::Deref};
use url::Url;

#[skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApubArticle {
    #[serde(rename = "type")]
    pub kind: ArticleType,
    pub id: ObjectId<ArticleWrapper>,
    pub attributed_to: ObjectId<PersonWrapper>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub cc: Vec<Url>,
    pub edits: CollectionId<EditCollection>,
    latest_version: EditVersion,
    content: String,
    name: String,
    protected: bool,
    pub(crate) media_type: Option<MediaTypeMarkdownOrHtml>,
    #[serde(deserialize_with = "deserialize_skip_error", default)]
    pub(crate) source: Option<Source>,
    published: Option<DateTime<Utc>>,
    updated: Option<DateTime<Utc>>,
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
        let latest_version = self.latest_edit_version(context)?;
        let wikibot = Person::wikibot(context)?;
        let local_instance: InstanceWrapper = Instance::read_local(context)?.into();
        Ok(ApubArticle {
            kind: Default::default(),
            id: self.ap_id.clone().into(),
            attributed_to: wikibot.ap_id.into(),
            to: vec![public(), local_instance.ap_id.clone().into()],
            cc: vec![],
            edits: self.edits_id()?.into(),
            latest_version,
            content: render_article_markdown(&self.text),
            name: self.title.clone(),
            protected: self.protected,
            media_type: Some(MediaTypeMarkdownOrHtml::Html),
            source: Some(Source::new(self.text.clone())),
            published: Some(self.published),
            updated: Some(self.updated),
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
        let mut iter = json.to.iter().merge(json.cc.iter());
        let instance = loop {
            if let Some(cid) = iter.next() {
                let cid = ObjectId::<InstanceWrapper>::from(cid.clone());
                if let Ok(c) = cid.dereference(context).await {
                    break c;
                }
            } else {
                Err(anyhow!("not found"))?;
            }
        };
        let text = read_from_string_or_source(&json.content, &json.media_type, &json.source);
        let form = DbArticleForm {
            title: json.name,
            text,
            ap_id: json.id.into(),
            local: false,
            instance_id: instance.id,
            protected: json.protected,
            updated: json.updated.or(json.published).unwrap_or_default(),
            pending: false,
        };
        validate_article_title(&form.title)?;
        let creator = json.attributed_to.dereference(context).await?;
        let article = Article::create_or_update(form, creator.id, context).await?;

        let mut edits = json.edits.dereference(&article, context).await?.0;
        edits.sort_by_key(|e| Reverse(e.published));

        Ok(article.into())
    }
}
