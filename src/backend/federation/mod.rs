use crate::backend::database::MyDataHandle;
use crate::config::IbisConfig;
use activitypub_federation::activity_sending::SendActivityTask;
use activitypub_federation::config::{Data, UrlVerifier};
use activitypub_federation::error::Error as ActivityPubError;
use activitypub_federation::protocol::context::WithContext;
use activitypub_federation::traits::{ActivityHandler, Actor};
use async_trait::async_trait;
use log::warn;
use serde::Serialize;
use std::fmt::Debug;
use url::Url;

pub mod activities;
pub mod objects;
pub mod routes;

pub async fn send_activity<Activity, ActorType: Actor>(
    actor: &ActorType,
    activity: Activity,
    recipients: Vec<Url>,
    data: &Data<MyDataHandle>,
) -> Result<(), <Activity as ActivityHandler>::Error>
where
    Activity: ActivityHandler + Serialize + Debug + Send + Sync,
    <Activity as ActivityHandler>::Error: From<activitypub_federation::error::Error>,
{
    let activity = WithContext::new_default(activity);
    let sends = SendActivityTask::prepare(&activity, actor, recipients, data).await?;
    for send in sends {
        let send = send.sign_and_send(data).await;
        if let Err(e) = send {
            warn!("Failed to send activity {:?}: {e}", activity);
        }
    }
    Ok(())
}

#[derive(Clone)]
pub struct VerifyUrlData(pub IbisConfig);

#[async_trait]
impl UrlVerifier for VerifyUrlData {
    /// Check domain against allowlist and blocklist from config file.
    async fn verify(&self, url: &Url) -> Result<(), ActivityPubError> {
        let domain = url.domain().unwrap();
        if let Some(allowlist) = &self.0.federation.allowlist {
            let allowlist = allowlist.split(',').collect::<Vec<_>>();
            if !allowlist.contains(&domain) {
                return Err(ActivityPubError::Other(format!(
                    "Domain {domain} is not allowed"
                )));
            }
        }
        if let Some(blocklist) = &self.0.federation.blocklist {
            let blocklist = blocklist.split(',').collect::<Vec<_>>();
            if blocklist.contains(&domain) {
                return Err(ActivityPubError::Other(format!(
                    "Domain {domain} is blocked"
                )));
            }
        }
        Ok(())
    }
}
