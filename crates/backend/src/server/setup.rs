use crate::utils::generate_article_ap_id;
use activitypub_federation::config::Data;
use chrono::Utc;
use ibis_database::{
    common::{
        MAIN_PAGE_NAME,
        article::{Article, EditVersion},
        instance::Instance,
        user::{LocalUserView, Person},
        utils::http_protocol_str,
    },
    error::BackendError,
    impls::{IbisContext, article::DbArticleForm, instance::DbInstanceForm},
    utils::generate_keypair,
};
use ibis_federate::{
    activities::submit_article_update,
    collections::{
        articles_collection::local_articles_url,
        instance_collection::linked_instances_url,
    },
};
use url::Url;

const MAIN_PAGE_DEFAULT_TEXT: &str = "Welcome to Ibis, the federated Wikipedia alternative!

This main page can only be edited by the admin. Use it as an introduction for new users, \
and to list interesting articles.
";

pub async fn setup(context: &Data<IbisContext>) -> Result<(), BackendError> {
    let domain = &context.conf.federation.domain;
    let ap_id = Url::parse(&format!("{}://{domain}", http_protocol_str()))?.into();
    let inbox_url = format!("{}://{domain}/inbox", http_protocol_str());
    let keypair = generate_keypair()?;
    let form = DbInstanceForm {
        domain: domain.to_string(),
        ap_id,
        articles_url: Some(local_articles_url(domain)?.into()),
        instances_url: linked_instances_url(domain)?.into(),
        inbox_url,
        public_key: keypair.public_key,
        private_key: Some(keypair.private_key),
        last_refreshed_at: Utc::now(),
        local: true,
        topic: None,
        name: None,
    };
    let instance = Instance::create(&form, context)?;

    let admin = LocalUserView::create(
        context.conf.setup.admin_username.clone(),
        Some(context.conf.setup.admin_password.clone()),
        true,
        None,
        context,
    )?;

    // Admin follows local instance by default
    Instance::follow(&admin.person.clone(), &instance, false, context)?;

    // Create the main page which is shown by default
    let form = DbArticleForm {
        title: MAIN_PAGE_NAME.to_string(),
        text: String::new(),
        ap_id: generate_article_ap_id(MAIN_PAGE_NAME, &instance)?,
        instance_id: instance.id,
        local: true,
        protected: true,
        updated: Utc::now(),
    };
    let article = Article::create(form, admin.person.id, context).await?;
    // also create an article so its included in most recently edited list
    submit_article_update(
        MAIN_PAGE_DEFAULT_TEXT.to_string(),
        "Default main page".to_string(),
        EditVersion::default(),
        &article,
        admin.person.into(),
        context,
    )
    .await?;

    // create ghost user
    Person::ghost(context)?;

    Ok(())
}
