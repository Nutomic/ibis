use crate::backend::database::article::DbArticleForm;
use crate::backend::database::instance::DbInstanceForm;
use crate::backend::database::MyData;
use crate::backend::error::MyResult;
use crate::backend::federation::routes::federation_routes;
use crate::backend::utils::generate_activity_id;
use crate::common::{DbArticle, DbInstance};
use crate::frontend::app::App;
use activitypub_federation::config::{FederationConfig, FederationMiddleware};
use activitypub_federation::fetch::collection_id::CollectionId;
use activitypub_federation::fetch::object_id::ObjectId;
use activitypub_federation::http_signatures::generate_actor_keypair;
use api::api_routes;
use axum::http::{HeaderValue, Request};
use axum::Server;
use axum::ServiceExt;
use axum::{middleware::Next, response::Response, Router};
use chrono::Local;
use diesel::Connection;
use diesel::PgConnection;
use diesel_migrations::embed_migrations;
use diesel_migrations::EmbeddedMigrations;
use diesel_migrations::MigrationHarness;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use log::info;
use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};
use tower::Layer;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

pub mod api;
pub mod database;
pub mod error;
pub mod federation;
mod utils;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

const FEDERATION_ROUTES_PREFIX: &str = "/federation_routes";

pub async fn start(hostname: &str, database_url: &str) -> MyResult<()> {
    let db_connection = Arc::new(Mutex::new(PgConnection::establish(database_url)?));
    db_connection
        .lock()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();

    let data = MyData { db_connection };
    let config = FederationConfig::builder()
        .domain(hostname)
        .app_data(data)
        .debug(true)
        .build()
        .await?;

    // Create local instance if it doesnt exist yet
    // TODO: Move this into setup api call
    if DbInstance::read_local_instance(&config.db_connection).is_err() {
        let ap_id = ObjectId::parse(&format!("http://{hostname}"))?;
        let articles_url = CollectionId::parse(&format!("http://{}/all_articles", hostname))?;
        let inbox_url = format!("http://{}/inbox", hostname);
        let keypair = generate_actor_keypair()?;
        let form = DbInstanceForm {
            ap_id,
            articles_url,
            inbox_url,
            public_key: keypair.public_key,
            private_key: Some(keypair.private_key),
            last_refreshed_at: Local::now().into(),
            local: true,
        };
        let instance = DbInstance::create(&form, &config.db_connection)?;

        // Create the main page which is shown by default
        let form = DbArticleForm {
            title: "Main_Page".to_string(),
            text: "Hello world!".to_string(),
            ap_id: ObjectId::parse(&format!("http://{hostname}/article/Main_Page"))?,
            instance_id: instance.id,
            local: true,
        };
        DbArticle::create(&form, &config.db_connection)?;
    }

    let conf = get_configuration(Some("Cargo.toml")).await.unwrap();
    let mut leptos_options = conf.leptos_options;
    let addr = hostname
        .to_socket_addrs()?
        .next()
        .expect("Failed to lookup domain name");
    leptos_options.site_addr = addr;
    let routes = generate_route_list(App);

    let config = config.clone();
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, || view! {  <App/> })
        .with_state(leptos_options)
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service(
            "/pkg/ibis.js",
            ServeFile::new_with_mime("assets/dist/ibis.js", &"application/javascript".parse()?),
        )
        .nest_service(
            "/pkg/ibis_bg.wasm",
            ServeFile::new_with_mime("assets/dist/ibis_bg.wasm", &"application/wasm".parse()?),
        )
        .nest(FEDERATION_ROUTES_PREFIX, federation_routes())
        .nest("/api/v1", api_routes())
        .layer(FederationMiddleware::new(config))
        .layer(CorsLayer::permissive());

    // Rewrite federation routes
    // https://docs.rs/axum/0.7.4/axum/middleware/index.html#rewriting-request-uri-in-middleware
    let middleware = axum::middleware::from_fn(federation_routes_middleware);
    let app_with_middleware = middleware.layer(app);

    info!("{addr}");
    Server::bind(&addr)
        .serve(app_with_middleware.into_make_service())
        .await?;

    Ok(())
}

/// Rewrite federation routes to use `FEDERATION_ROUTES_PREFIX`, to avoid conflicts
/// with frontend routes. If a request is an Activitypub fetch as indicated by
/// `Accept: application/activity+json` header, use the federation routes. Otherwise
/// leave the path unchanged so it can go to frontend.
async fn federation_routes_middleware<B>(request: Request<B>, next: Next<B>) -> Response {
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
    if uri.ends_with('/') {
        uri.pop();
    }
    parts.uri = uri.parse().unwrap();
    let request = Request::from_parts(parts, body);

    next.run(request).await
}
