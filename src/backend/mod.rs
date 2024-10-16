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
        DbArticle,
        DbInstance,
        DbPerson,
        EditVersion,
        MAIN_PAGE_NAME,
    },
    frontend::app::App,
};
use activitypub_federation::{
    config::{Data, FederationConfig, FederationMiddleware},
    fetch::{collection_id::CollectionId, object_id::ObjectId},
    http_signatures::generate_actor_keypair,
};
use api::api_routes;
use assets::asset_routes;
use axum::{
    body::Body,
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
    Router,
    ServiceExt,
};
use axum_macros::debug_middleware;
use chrono::Local;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use leptos::leptos_config::get_config_from_str;
use leptos_axum::{generate_route_list, LeptosRoutes};
use log::info;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_layer::Layer;

pub mod api;
mod assets;
pub mod config;
pub mod database;
pub mod error;
pub mod federation;
mod nodeinfo;
mod utils;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

const FEDERATION_ROUTES_PREFIX: &str = "/federation_routes";

pub async fn start(config: IbisConfig) -> MyResult<()> {
    let manager = ConnectionManager::<PgConnection>::new(&config.database.connection_url);
    let db_pool = Pool::builder()
        .max_size(config.database.pool_size)
        .build(manager)?;

    db_pool
        .get()?
        .run_pending_migrations(MIGRATIONS)
        .expect("run migrations");
    let data = IbisData { db_pool, config };
    let data = FederationConfig::builder()
        .domain(data.config.federation.domain.clone())
        .url_verifier(Box::new(VerifyUrlData(data.config.clone())))
        .app_data(data)
        .debug(true)
        .build()
        .await?;

    // Create local instance if it doesnt exist yet
    if DbInstance::read_local_instance(&data).is_err() {
        setup(&data.to_request_data()).await?;
    }

    let mut conf = get_config_from_str(include_str!("../../Cargo.toml"))?;
    conf.site_addr = data.config.bind;
    let routes = generate_route_list(App);

    let config = data.clone();
    let app = Router::new()
        .leptos_routes(&conf, routes, App)
        .with_state(conf)
        .nest("", asset_routes()?)
        .nest(FEDERATION_ROUTES_PREFIX, federation_routes())
        .nest("/api/v1", api_routes())
        .nest("", nodeinfo::config())
        .layer(FederationMiddleware::new(config))
        .layer(CorsLayer::permissive());

    // Rewrite federation routes
    // https://docs.rs/axum/0.7.4/axum/middleware/index.html#rewriting-request-uri-in-middleware
    let middleware = axum::middleware::from_fn(federation_routes_middleware);
    let app_with_middleware = middleware.layer(app);

    info!("Listening on {}", &data.config.bind);
    let listener = TcpListener::bind(&data.config.bind).await?;
    axum::serve(listener, app_with_middleware.into_make_service()).await?;

    Ok(())
}

const MAIN_PAGE_DEFAULT_TEXT: &str = "Welcome to Ibis, the federated Wikipedia alternative!

This main page can only be edited by the admin. Use it as an introduction for new users, \
and to list interesting articles.";

async fn setup(data: &Data<IbisData>) -> Result<(), Error> {
    let domain = &data.config.federation.domain;
    let ap_id = ObjectId::parse(&format!("{}://{domain}", http_protocol_str()))?;
    let articles_url =
        CollectionId::parse(&format!("{}://{domain}/all_articles", http_protocol_str()))?;
    let inbox_url = format!("{}://{domain}/inbox", http_protocol_str());
    let keypair = generate_actor_keypair()?;
    let form = DbInstanceForm {
        domain: domain.to_string(),
        ap_id,
        description: Some("New Ibis instance".to_string()),
        articles_url,
        inbox_url,
        public_key: keypair.public_key,
        private_key: Some(keypair.private_key),
        last_refreshed_at: Local::now().into(),
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

    Ok(())
}

/// Rewrite federation routes to use `FEDERATION_ROUTES_PREFIX`, to avoid conflicts
/// with frontend routes. If a request is an Activitypub fetch as indicated by
/// `Accept: application/activity+json` header, use the federation routes. Otherwise
/// leave the path unchanged so it can go to frontend.
#[debug_middleware]
async fn federation_routes_middleware(request: Request<Body>, next: Next) -> Response {
    let (mut parts, body) = request.into_parts();
    // rewrite uri based on accept header
    let mut uri = parts.uri.to_string();
    let accept_value = HeaderValue::from_static("application/activity+json");
    if Some(&accept_value) == parts.headers.get("Accept")
        || Some(&accept_value) == parts.headers.get("Content-Type")
    {
        uri = format!("{FEDERATION_ROUTES_PREFIX}{uri}");
    }
    // drop trailing slash
    if uri.ends_with('/') && uri.len() > 1 {
        uri.pop();
    }
    parts.uri = uri
        .parse()
        .expect("can parse uri after dropping trailing slash");
    let request = Request::from_parts(parts, body);

    next.run(request).await
}
