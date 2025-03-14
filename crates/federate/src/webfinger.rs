use activitypub_federation::{
    config::Data,
    fetch::webfinger::{WEBFINGER_CONTENT_TYPE, Webfinger, WebfingerLink, extract_webfinger_name},
};
use axum::{
    Json,
    Router,
    extract::Query,
    http::header::CONTENT_TYPE,
    response::IntoResponse,
    routing::get,
};
use axum_macros::debug_handler;
use ibis_database::{common::user::Person, error::BackendResult, impls::IbisContext};
use serde::Deserialize;
use url::Url;

pub fn config() -> Router<()> {
    Router::new().route("/.well-known/webfinger", get(get_webfinger_response))
}

#[derive(Deserialize)]
struct Params {
    resource: String,
}

#[debug_handler]
async fn get_webfinger_response(
    info: Query<Params>,
    context: Data<IbisContext>,
) -> BackendResult<impl IntoResponse> {
    let name = extract_webfinger_name(&info.resource, &context)?;

    let links = if name == context.conf.federation.domain {
        // webfinger response for instance actor (required for mastodon authorized fetch)
        let url = Url::parse(&format!("https://{}", context.conf.federation.domain))?;
        webfinger_link_for_actor(url)?
    } else {
        let user_id: Url = Person::read_from_name(name, &None, &context)?.ap_id.into();
        webfinger_link_for_actor(user_id)?
    };

    let webfinger = Webfinger {
        subject: info.resource.clone(),
        links,
        ..Default::default()
    };

    Ok((
        [(CONTENT_TYPE, WEBFINGER_CONTENT_TYPE.clone())],
        Json(webfinger),
    ))
}

fn webfinger_link_for_actor(url: Url) -> BackendResult<Vec<WebfingerLink>> {
    let vec = vec![
        WebfingerLink {
            rel: Some("http://webfinger.net/rel/profile-page".into()),
            kind: Some("text/html".into()),
            href: Some(url.clone()),
            ..Default::default()
        },
        WebfingerLink {
            rel: Some("self".into()),
            kind: Some("application/activity+json".into()),
            href: Some(url),
            ..Default::default()
        },
    ];
    Ok(vec)
}
