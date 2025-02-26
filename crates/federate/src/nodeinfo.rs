use activitypub_federation::config::Data;
use axum::{Json, Router, routing::get};
use ibis_database::{
    common::utils::http_protocol_str,
    error::BackendResult,
    impls::{IbisContext, instance_stats::InstanceStats},
};
use serde::Serialize;
use url::Url;

pub fn config() -> Router<()> {
    Router::new()
        .route("/nodeinfo/2.1.json", get(node_info))
        .route("/.well-known/nodeinfo", get(node_info_well_known))
}

async fn node_info_well_known(
    context: Data<IbisContext>,
) -> BackendResult<Json<NodeInfoWellKnown>> {
    Ok(Json(NodeInfoWellKnown {
        links: vec![NodeInfoWellKnownLinks {
            rel: Url::parse("http://nodeinfo.diaspora.software/ns/schema/2.1")?,
            href: Url::parse(&format!(
                "{}://{}/nodeinfo/2.1.json",
                http_protocol_str(),
                context.domain()
            ))?,
        }],
    }))
}

async fn node_info(context: Data<IbisContext>) -> BackendResult<Json<NodeInfo>> {
    let stats = InstanceStats::read(&context)?;
    Ok(Json(NodeInfo {
        version: "2.1".to_string(),
        software: NodeInfoSoftware {
            name: "ibis".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            repository: "https://github.com/Nutomic/ibis".to_string(),
            homepage: "https://ibis.wiki/".to_string(),
        },
        protocols: vec!["activitypub".to_string()],
        usage: NodeInfoUsage {
            users: NodeInfoUsers {
                total: stats.users,
                active_month: stats.users_active_month,
                active_halfyear: stats.users_active_half_year,
            },
            local_posts: stats.articles,
            local_comments: stats.comments,
        },
        open_registrations: context.config.options.registration_open,
        services: Default::default(),
        metadata: vec![],
    }))
}

#[derive(Serialize)]
struct NodeInfoWellKnown {
    pub links: Vec<NodeInfoWellKnownLinks>,
}

#[derive(Serialize)]
struct NodeInfoWellKnownLinks {
    pub rel: Url,
    pub href: Url,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
    pub version: String,
    pub software: NodeInfoSoftware,
    pub protocols: Vec<String>,
    pub usage: NodeInfoUsage,
    pub open_registrations: bool,
    /// These fields are required by the spec for no reason
    pub services: NodeInfoServices,
    pub metadata: Vec<String>,
}

#[derive(Serialize)]
pub struct NodeInfoSoftware {
    pub name: String,
    pub version: String,
    pub repository: String,
    pub homepage: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfoUsage {
    pub users: NodeInfoUsers,
    pub local_posts: i32,
    pub local_comments: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfoUsers {
    pub total: i32,
    pub active_month: i32,
    pub active_halfyear: i32,
}

#[derive(Serialize, Default)]
pub struct NodeInfoServices {
    pub inbound: Vec<String>,
    pub outbound: Vec<String>,
}
