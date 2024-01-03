use crate::backend::database::instance::{DbInstance, DbInstanceForm};
use crate::backend::database::MyData;
use crate::backend::error::MyResult;
use crate::backend::federation::routes::federation_routes;
use crate::backend::utils::generate_activity_id;
use activitypub_federation::config::{FederationConfig, FederationMiddleware};
use activitypub_federation::fetch::collection_id::CollectionId;
use activitypub_federation::fetch::object_id::ObjectId;
use activitypub_federation::http_signatures::generate_actor_keypair;
use api::api_routes;
use axum::{Router, Server};
use chrono::Local;
use diesel::Connection;
use diesel::PgConnection;
use diesel_migrations::embed_migrations;
use diesel_migrations::EmbeddedMigrations;
use diesel_migrations::MigrationHarness;
use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use log::info;
use leptos_axum::{generate_route_list, LeptosRoutes};
use leptos::*;use leptos_meta::*;use leptos_router::*;

pub mod api;
pub mod database;
pub mod error;
pub mod federation;
mod utils;


const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

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

    // TODO: Move this into setup api call
    let ap_id = ObjectId::parse(&format!("http://{}", hostname))?;
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
    DbInstance::create(&form, &config.db_connection)?;

    let conf = get_configuration(Some("Cargo.toml")).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    info!("Listening with axum on {hostname}");
    let config = config.clone();
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, || view! {  <App/> } )
        .with_state(leptos_options)
        .nest("", federation_routes())
        .nest("/api/v1", api_routes())
        .layer(FederationMiddleware::new(config))
        .layer(CorsLayer::permissive());

    /*
    let addr = hostname
        .to_socket_addrs()?
        .next()
        .expect("Failed to lookup domain name");
     */
    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <>
            <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
            <Stylesheet id="leptos" href="/pkg/ibis.css"/>
            <Meta name="description" content="Leptos implementation of a HackerNews demo."/>
        test
        </>
    }
}