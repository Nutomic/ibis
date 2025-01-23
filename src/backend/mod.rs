use crate::{
    backend::{
        config::IbisConfig,
        database::{article::DbArticleForm, instance::DbInstanceForm, IbisContext},
        federation::{activities::submit_article_update, VerifyUrlData},
        utils::{
            error::{Error, MyResult},
            generate_activity_id,
        },
    },
    common::{
        article::{DbArticle, EditVersion},
        instance::DbInstance,
        user::DbPerson,
        utils::http_protocol_str,
        MAIN_PAGE_NAME,
    },
};
use activitypub_federation::{
    config::{Data, FederationConfig},
    fetch::object_id::ObjectId,
};
use chrono::Utc;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use federation::objects::{
    articles_collection::local_articles_url,
    instance_collection::linked_instances_url,
};
use log::info;
use server::start_server;
use std::{net::SocketAddr, thread};
use tokio::sync::oneshot;
use utils::{generate_keypair, scheduled_tasks};

pub mod api;
pub mod config;
pub mod database;
pub mod federation;
mod server;
pub mod utils;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub async fn start(
    config: IbisConfig,
    override_hostname: Option<SocketAddr>,
    notify_start: Option<oneshot::Sender<()>>,
) -> MyResult<()> {
    let manager = ConnectionManager::<PgConnection>::new(&config.database.connection_url);
    let db_pool = Pool::builder()
        .max_size(config.database.pool_size)
        .build(manager)?;

    db_pool
        .get()?
        .run_pending_migrations(MIGRATIONS)
        .expect("run migrations");
    let context = IbisContext { db_pool, config };
    let data = FederationConfig::builder()
        .domain(context.config.federation.domain.clone())
        .url_verifier(Box::new(VerifyUrlData(context.config.clone())))
        .app_data(context)
        .http_fetch_limit(1000)
        .debug(cfg!(debug_assertions))
        .build()
        .await?;

    if DbInstance::read_local(&data).is_err() {
        info!("Running setup for new instance");
        setup(&data.to_request_data()).await?;
    }

    let db_pool = data.db_pool.clone();
    thread::spawn(move || {
        scheduled_tasks::start(db_pool);
    });

    start_server(data, override_hostname, notify_start).await?;

    Ok(())
}

const MAIN_PAGE_DEFAULT_TEXT: &str = "Welcome to Ibis, the federated Wikipedia alternative!

This main page can only be edited by the admin. Use it as an introduction for new users, \
and to list interesting articles.";

async fn setup(context: &Data<IbisContext>) -> Result<(), Error> {
    let domain = &context.config.federation.domain;
    let ap_id = ObjectId::parse(&format!("{}://{domain}", http_protocol_str()))?;
    let inbox_url = format!("{}://{domain}/inbox", http_protocol_str());
    let keypair = generate_keypair()?;
    let form = DbInstanceForm {
        domain: domain.to_string(),
        ap_id,
        articles_url: Some(local_articles_url(domain)?),
        instances_url: Some(linked_instances_url(domain)?),
        inbox_url,
        public_key: keypair.public_key,
        private_key: Some(keypair.private_key),
        last_refreshed_at: Utc::now(),
        local: true,
        topic: None,
        name: None,
    };
    let instance = DbInstance::create(&form, context)?;

    let person = DbPerson::create_local(
        context.config.setup.admin_username.clone(),
        context.config.setup.admin_password.clone(),
        true,
        context,
    )?;

    // Create the main page which is shown by default
    let form = DbArticleForm {
        title: MAIN_PAGE_NAME.to_string(),
        text: String::new(),
        ap_id: ObjectId::parse(&format!(
            "{}://{domain}/article/{MAIN_PAGE_NAME}",
            http_protocol_str()
        ))?,
        instance_id: instance.id,
        local: true,
        protected: true,
        approved: true,
    };
    let article = DbArticle::create(form, context)?;
    // also create an article so its included in most recently edited list
    submit_article_update(
        MAIN_PAGE_DEFAULT_TEXT.to_string(),
        "Default main page".to_string(),
        EditVersion::default(),
        &article,
        person.person.id,
        context,
    )
    .await?;

    // create ghost user
    DbPerson::ghost(context)?;

    Ok(())
}
