use crate::{
    backend::{api::user::validate, database::IbisContext},
    common::{Auth, AUTH_COOKIE},
};
use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};
use axum_macros::debug_middleware;
use http::{header::COOKIE, HeaderValue};
use std::{collections::HashSet, sync::Arc};

pub(super) const FEDERATION_ROUTES_PREFIX: &str = "/federation_routes";

/// Checks all headers and cookies (including duplicates) for first valid auth token.
/// We need to extract cookies manually because CookieJar ignores duplicates.
/// If user is authenticated sets extensions `Auth` and `LocalUserView`.
#[debug_middleware]
pub(super) async fn auth_middleware(
    State(context): State<Arc<IbisContext>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let headers = request.headers();
    let cookies = headers
        .get(COOKIE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default()
        .split(';')
        .flat_map(|s| s.split_once('='))
        .filter(|s| s.0.trim() == AUTH_COOKIE)
        .map(|s| s.1);
    let headers = headers
        .get_all(AUTH_COOKIE)
        .into_iter()
        .filter_map(|h| h.to_str().ok());
    let auth: HashSet<_> = headers.chain(cookies).map(|s| s.to_string()).collect();

    for auth in auth {
        if let Ok(local_user) = validate(&auth, &context).await {
            request.extensions_mut().insert(Auth(Some(auth)));
            request.extensions_mut().insert(local_user);
        }
    }
    next.run(request).await
}

/// Rewrite federation routes to use `FEDERATION_ROUTES_PREFIX`, to avoid conflicts
/// with frontend routes. If a request is an Activitypub fetch as indicated by
/// `Accept: application/activity+json` header, use the federation routes. Otherwise
/// leave the path unchanged so it can go to frontend.
#[debug_middleware]
pub(super) async fn federation_routes_middleware(request: Request<Body>, next: Next) -> Response {
    let (mut parts, body) = request.into_parts();
    // rewrite uri based on accept header
    let mut uri = parts.uri.to_string();
    const VALUE1: HeaderValue = HeaderValue::from_static("application/activity+json");
    const VALUE2: HeaderValue = HeaderValue::from_static(
        r#"application/ld+json; profile="https://www.w3.org/ns/activitystreams""#,
    );
    let accept = parts.headers.get("Accept");
    let content_type = parts.headers.get("Content-Type");
    if Some(&VALUE1) == accept
        || Some(&VALUE2) == accept
        || Some(&VALUE1) == content_type
        || Some(&VALUE2) == content_type
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
