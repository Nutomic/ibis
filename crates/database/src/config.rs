use crate::{common::instance::Options, error::BackendResult};
use anyhow::anyhow;
use config::Config;
use doku::Document;
use serde::Deserialize;
use smart_default::SmartDefault;
use url::Url;

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Document, SmartDefault)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct IbisConfig {
    /// Details about the PostgreSQL database connection
    pub database: IbisConfigDatabase,
    /// Details of the initial admin account
    pub setup: IbisConfigSetup,
    /// Domain for HTTP and frontend
    pub domain: String,
    pub federation: IbisConfigFederation,
    pub options: Options,
    pub email: Option<IbisConfigEmail>,
    pub oauth_providers: Vec<OAuthProvider>,
}

impl IbisConfig {
    pub fn read() -> BackendResult<Self> {
        let config_file = if cfg!(test) {
            "../../config.toml"
        } else {
            "config.toml"
        };
        let config: Self = Config::builder()
            .add_source(config::File::with_name(config_file))
            // Cant use _ as separator due to https://github.com/mehcode/config-rs/issues/391
            .add_source(config::Environment::with_prefix("IBIS").separator("__"))
            .build()?
            .try_deserialize()?;

        if config.options.email_required && config.email.is_none() {
            return Err(anyhow!("Email is required but no email send config provided").into());
        }
        Ok(config)
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Document, SmartDefault)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct IbisConfigDatabase {
    /// Database connection url
    #[default("postgres://ibis:password@localhost:5432/ibis")]
    #[doku(example = "postgres://ibis:password@localhost:5432/ibis")]
    pub connection_url: String,
    /// Database connection pool size
    #[default(30)]
    #[doku(example = "30")]
    pub pool_size: u32,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Document)]
#[serde(deny_unknown_fields)]
pub struct IbisConfigEmail {
    /// Connection parameters for email transport
    /// https://docs.rs/lettre/0.11.14/lettre/transport/smtp/struct.AsyncSmtpTransport.html#method.from_url
    #[doku(example = "smtps://user:pass@hostname:port")]
    pub connection_url: String,
    /// Sender address for email sent by ibis
    #[doku(example = "ibis@example.com")]
    pub from_address: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Document, SmartDefault)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct IbisConfigSetup {
    /// Username for the admin account
    #[default("ibis")]
    #[doku(example = "ibis")]
    pub admin_username: String,
    /// Initial password for admin account (can be changed later)
    #[default("ibis")]
    #[doku(example = "ibis")]
    pub admin_password: String,
    /// Name of the Activitypub group which is used to federate articles
    #[default("wiki")]
    #[doku(example = "wiki")]
    pub group_name: String,
    /// Name of the bot account used to federate articles
    #[default("wikibot")]
    #[doku(example = "wikibot")]
    pub wiki_bot_name: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Document, SmartDefault)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct IbisConfigFederation {
    /// Domain used for federation
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

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Document)]
#[serde(deny_unknown_fields)]
/// oauth provider with client_secret - should never be sent to the client
pub struct OAuthProvider {
    /// The OAuth 2.0 provider name displayed to the user on the Login page
    pub display_name: String,
    /// The issuer url of the OAUTH provider.
    pub issuer: Url,
    /// The authorization endpoint is used to interact with the resource owner and obtain an
    /// authorization grant. This is usually provided by the OAUTH provider.
    pub authorization_endpoint: Url,
    /// The token endpoint is used by the client to obtain an access token by presenting its
    /// authorization grant or refresh token. This is usually provided by the OAUTH provider.
    pub token_endpoint: Url,
    /// The UserInfo Endpoint is an OAuth 2.0 Protected Resource that returns Claims about the
    /// authenticated End-User. This is defined in the OIDC specification.
    pub userinfo_endpoint: Url,
    /// The client_id is provided by the OAuth 2.0 provider and is a unique identifier to this
    /// service
    pub client_id: String,
    /// The client_secret is provided by the OAuth 2.0 provider and is used to authenticate this
    /// service with the provider
    pub client_secret: String,
    /// Lists the scopes requested from users. Users will have to grant access to the requested scope
    /// at sign up.
    pub scopes: String,
}
