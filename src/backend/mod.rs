use crate::{
    backend::{
        config::IbisConfig,
        database::{article::DbArticleForm, instance::DbInstanceForm, IbisData},
        error::{Error, MyResult},
        federation::{activities::submit_article_update, routes::federation_routes, VerifyUrlData},
        utils::generate_activity_id,
    },
    common::{
        utils::http_protocol_str,
        Auth,
        DbArticle,
        DbInstance,
        DbPerson,
        EditVersion,
        MAIN_PAGE_NAME,
    },
    frontend::app::{shell, App},
};
use activitypub_federation::{
    config::{Data, FederationConfig, FederationMiddleware},
    fetch::object_id::ObjectId,
};
use api::api_routes;
use assets::file_and_error_handler;
use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::get,
    Extension,
    Router,
    ServiceExt,
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
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use log::info;
use middleware::{auth_middleware, federation_routes_middleware, FEDERATION_ROUTES_PREFIX};
use std::{net::SocketAddr, sync::Arc, thread};
use tokio::{net::TcpListener, sync::oneshot};
use tower_http::{compression::CompressionLayer, cors::CorsLayer};
use tower_layer::Layer;
use utils::generate_keypair;

pub mod api;
mod assets;
pub mod config;
pub mod database;
pub mod error;
pub mod federation;
mod middleware;
mod nodeinfo;
mod scheduled_tasks;
mod utils;

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
    let ibis_data = IbisData { db_pool, config };
    let data = FederationConfig::builder()
        .domain(ibis_data.config.federation.domain.clone())
        .url_verifier(Box::new(VerifyUrlData(ibis_data.config.clone())))
        .app_data(ibis_data.clone())
        .http_fetch_limit(1000)
        .debug(cfg!(debug_assertions))
        .build()
        .await?;

    // Create local instance if it doesnt exist yet
    if DbInstance::read_local_instance(&data).is_err() {
        setup(&data.to_request_data()).await?;
    }

    let db_pool = data.db_pool.clone();
    thread::spawn(move || {
        scheduled_tasks::start(db_pool);
    });

    let leptos_options = get_config_from_str(include_str!("../../Cargo.toml"))?;
    let mut addr = leptos_options.site_addr;
    if let Some(override_hostname) = override_hostname {
        addr = override_hostname;
    }
    let routes = generate_route_list(App);

    let config = data.clone();
    let arc_data = Arc::new(ibis_data);
    let app = Router::new()
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .fallback(file_and_error_handler)
        .with_state(leptos_options)
        .nest(FEDERATION_ROUTES_PREFIX, federation_routes())
        .nest("/api/v1", api_routes())
        .nest("", nodeinfo::config())
        .layer(FederationMiddleware::new(config))
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .route_layer(from_fn_with_state(arc_data.clone(), auth_middleware));

    // Rewrite federation routes
    // https://docs.rs/axum/0.7.4/axum/middleware/index.html#rewriting-request-uri-in-middleware
    let middleware = axum::middleware::from_fn(federation_routes_middleware);
    let app_with_middleware = middleware.layer(app);

    info!("Listening on {}", &addr);
    let listener = TcpListener::bind(&addr).await?;
    if let Some(notify_start) = notify_start {
        notify_start.send(()).expect("send oneshot");
    }
    axum::serve(listener, app_with_middleware.into_make_service()).await?;

    Ok(())
}

/// Make auth token available in hydrate mode
async fn leptos_routes_handler(
    auth: Option<Extension<Auth>>,
    State(leptos_options): State<LeptosOptions>,
    request: Request<Body>,
) -> Response {
    let handler = leptos_axum::render_app_async_with_context(
        move || {
            if let Some(auth) = &auth {
                provide_context(auth.0.clone());
            }
        },
        move || shell(leptos_options.clone()),
    );

    handler(request).await.into_response()
}

const MAIN_PAGE_DEFAULT_TEXT: &str = "Welcome to Ibis, the federated Wikipedia alternative!

This main page can only be edited by the admin. Use it as an introduction for new users, \
and to list interesting articles.";

async fn setup(data: &Data<IbisData>) -> Result<(), Error> {
    let domain = &data.config.federation.domain;
    let ap_id = ObjectId::parse(&format!("{}://{domain}", http_protocol_str()))?;
    let inbox_url = format!("{}://{domain}/inbox", http_protocol_str());
    let keypair = generate_keypair()?;
    let form = DbInstanceForm {
        domain: domain.to_string(),
        ap_id,
        description: Some("New Ibis instance".to_string()),
        articles_url: Some(local_articles_url(domain)?),
        instances_url: Some(linked_instances_url(domain)?),
        inbox_url,
        public_key: keypair.public_key,
        private_key: Some(keypair.private_key),
        last_refreshed_at: Utc::now(),
        local: true,
    };
    let instance = DbInstance::create(&form, data)?;

    let person = DbPerson::create_local(
        data.config.setup.admin_username.clone(),
        data.config.setup.admin_password.clone(),
        true,
        data,
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
    let article = DbArticle::create(form, data)?;
    // also create an article so its included in most recently edited list
    submit_article_update(
        MAIN_PAGE_DEFAULT_TEXT.to_string(),
        "Default main page".to_string(),
        EditVersion::default(),
        &article,
        person.person.id,
        data,
    )
    .await?;

    // create ghost user
    DbPerson::ghost(data)?;

    Ok(())
}
