use super::{database::IbisData, utils::error::MyResult};
use crate::{
    backend::{api::api_routes, federation::routes::federation_routes},
    common::Auth,
    frontend::app::{shell, App},
};
use activitypub_federation::config::{FederationConfig, FederationMiddleware};
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
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use log::info;
use middleware::{auth_middleware, federation_routes_middleware, FEDERATION_ROUTES_PREFIX};
use std::{net::SocketAddr, ops::Deref, sync::Arc};
use tokio::{net::TcpListener, sync::oneshot};
use tower_http::{compression::CompressionLayer, cors::CorsLayer};
use tower_layer::Layer;

mod assets;
mod middleware;
mod nodeinfo;

pub(super) async fn start_server(
    data: FederationConfig<IbisData>,
    override_hostname: Option<SocketAddr>,
    notify_start: Option<oneshot::Sender<()>>,
) -> MyResult<()> {
    let leptos_options = get_config_from_str(include_str!("../../../Cargo.toml"))?;
    let mut addr = leptos_options.site_addr;
    if let Some(override_hostname) = override_hostname {
        addr = override_hostname;
    }
    let routes = generate_route_list(App);

    let arc_data = Arc::new(data.deref().clone());
    let app = Router::new()
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .fallback(file_and_error_handler)
        .with_state(leptos_options)
        .nest(FEDERATION_ROUTES_PREFIX, federation_routes())
        .nest("/api/v1", api_routes())
        .nest("", nodeinfo::config())
        .layer(FederationMiddleware::new(data))
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .route_layer(from_fn_with_state(arc_data, auth_middleware));

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
