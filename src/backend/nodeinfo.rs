use crate::{
    backend::{database::IbisData, error::MyResult},
    common::utils::http_protocol_str,
};
use activitypub_federation::config::Data;
use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use url::Url;

pub fn config() -> Router<()> {
    Router::new()
        .route("/nodeinfo/2.0.json", get(node_info))
        .route("/.well-known/nodeinfo", get(node_info_well_known))
}

async fn node_info_well_known(data: Data<IbisData>) -> MyResult<Json<NodeInfoWellKnown>> {
    Ok(Json(NodeInfoWellKnown {
        links: vec![NodeInfoWellKnownLinks {
            rel: Url::parse("http://nodeinfo.diaspora.software/ns/schema/2.0")?,
            href: Url::parse(&format!(
                "{}://{}/nodeinfo/2.0.json",
                http_protocol_str(),
                data.domain()
            ))?,
        }],
    }))
}

async fn node_info(data: Data<IbisData>) -> MyResult<Json<NodeInfo>> {
    Ok(Json(NodeInfo {
        version: "2.0".to_string(),
        software: NodeInfoSoftware {
            name: "ibis".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        protocols: vec!["activitypub".to_string()],
        open_registrations: data.config.options.registration_open,
    }))
}

#[derive(Serialize, Deserialize, Debug)]
struct NodeInfoWellKnown {
    pub links: Vec<NodeInfoWellKnownLinks>,
}

#[derive(Serialize, Deserialize, Debug)]
struct NodeInfoWellKnownLinks {
    pub rel: Url,
    pub href: Url,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NodeInfo {
    pub version: String,
    pub software: NodeInfoSoftware,
    pub protocols: Vec<String>,
    pub open_registrations: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct NodeInfoSoftware {
    pub name: String,
    pub version: String,
}
