use crate::backend::error::MyResult;
use config::Config;
use doku::Document;
use serde::Deserialize;
use smart_default::SmartDefault;
use std::net::SocketAddr;

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Document, SmartDefault)]
#[serde(default)]
pub struct IbisConfig {
    /// Address where ibis should listen for incoming requests
    #[default("127.0.0.1:8081".parse().expect("parse config bind"))]
    #[doku(as = "String", example = "127.0.0.1:8081")]
    pub bind: SocketAddr,
    /// Details about the PostgreSQL database connection
    pub database: IbisConfigDatabase,
    /// Whether users can create new accounts
    #[default = true]
    #[doku(example = "true")]
    pub registration_open: bool,
    /// Details of the initial admin account
    pub setup: IbisConfigSetup,
    pub federation: IbisConfigFederation,
}

impl IbisConfig {
    pub fn read() -> MyResult<Self> {
        let config = Config::builder()
            .add_source(config::File::with_name("config.toml"))
            // Cant use _ as separator due to https://github.com/mehcode/config-rs/issues/391
            .add_source(config::Environment::with_prefix("IBIS").separator("__"))
            .build()?;

        Ok(config.try_deserialize()?)
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Document, SmartDefault)]
#[serde(default)]
pub struct IbisConfigDatabase {
    /// Database connection url
    #[default("postgres://ibis:password@localhost:5432/ibis")]
    #[doku(example = "postgres://ibis:password@localhost:5432/ibis")]
    pub connection_url: String,
    /// Database connection pool size
    #[default(5)]
    #[doku(example = "5")]
    pub pool_size: u32,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Document, SmartDefault)]
#[serde(default)]
pub struct IbisConfigSetup {
    #[default("ibis")]
    #[doku(example = "ibis")]
    pub admin_username: String,
    #[default("ibis")]
    #[doku(example = "ibis")]
    pub admin_password: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Document, SmartDefault)]
#[serde(default)]
pub struct IbisConfigFederation {
    /// Domain name of the instance, mandatory for federation
    #[default("example.com")]
    #[doku(example = "example.com")]
    pub domain: String,
    /// Comma separated list of instances which are allowed for federation. If set, federation
    /// with other domains is blocked
    #[default(None)]
    #[doku(example = "good.com,friends.org")]
    pub allowlist: Option<String>,
    /// Comma separated list of instances which are blocked for federation
    #[default(None)]
    #[doku(example = "evil.com,bad.org")]
    pub blocklist: Option<String>,
}
