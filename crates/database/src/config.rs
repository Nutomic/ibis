use crate::{common::instance::Options, error::BackendResult};
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
    pub federation: IbisConfigFederation,
    pub options: Options,
    pub oauth_providers: Vec<OAuthProvider>,
}

impl IbisConfig {
    pub fn read() -> BackendResult<Self> {
        let config_file = if cfg!(test) {
            "../../config.toml"
        } else {
            "config.toml"
        };
        let config = Config::builder()
            .add_source(config::File::with_name(config_file))
            // Cant use _ as separator due to https://github.com/mehcode/config-rs/issues/391
            .add_source(config::Environment::with_prefix("IBIS").separator("__"))
            .build()?;

        Ok(config.try_deserialize()?)
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

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Document, SmartDefault)]
#[serde(default)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
    /// The OAuth 2.0 claim containing the unique user ID returned by the provider. Usually this
    /// should be set to "sub".
    pub id_claim: String,
    /// The client_id is provided by the OAuth 2.0 provider and is a unique identifier to this
    /// service
    pub client_id: String,
    /// The client_secret is provided by the OAuth 2.0 provider and is used to authenticate this
    /// service with the provider
    pub client_secret: String,
    /// Lists the scopes requested from users. Users will have to grant access to the requested scope
    /// at sign up.
    pub scopes: String,
    /// Automatically sets email as verified on registration
    pub auto_verify_email: bool,
    /// Allows linking an OAUTH account to an existing user account by matching emails
    pub account_linking_enabled: bool,
    /// switch to enable or disable an oauth provider
    pub enabled: bool,
    /// switch to enable or disable PKCE
    pub use_pkce: bool,
}
