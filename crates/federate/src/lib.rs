use activities::announce::AnnounceActivity;
use activitypub_federation::{
    activity_queue::queue_activity,
    config::{Data, UrlVerifier},
    error::Error as ActivityPubError,
    protocol::context::WithContext,
    traits::{ActivityHandler, Actor},
};
use async_trait::async_trait;
use ibis_database::{
    common::utils::http_protocol_str,
    config::IbisConfig,
    error::BackendResult,
    impls::IbisContext,
};
use objects::{instance::InstanceWrapper, user::PersonWrapper};
use rand::{Rng, distributions::Alphanumeric, thread_rng};
use routes::AnnouncableActivities;
use serde::Serialize;
use std::fmt::Debug;
use url::Url;

pub mod activities;
pub mod nodeinfo;
pub mod objects;
pub mod routes;
pub mod validate;

pub async fn send_activity<Activity, ActorType: Actor>(
    actor: &ActorType,
    activity: Activity,
    recipients: Vec<Url>,
    context: &Data<IbisContext>,
) -> Result<(), <Activity as ActivityHandler>::Error>
where
    Activity: ActivityHandler + Serialize + Debug + Send + Sync,
    <Activity as ActivityHandler>::Error: From<activitypub_federation::error::Error>,
{
    let activity = WithContext::new_default(activity);
    queue_activity(&activity, actor, recipients, context).await?;
    Ok(())
}

pub async fn send_activity_to_instance(
    actor: &PersonWrapper,
    activity: AnnouncableActivities,
    instance: &InstanceWrapper,
    context: &Data<IbisContext>,
) -> BackendResult<()> {
    if instance.local {
        AnnounceActivity::send(activity, context).await?;
    } else {
        let inbox_url = instance.inbox_url.parse()?;
        send_activity(actor, activity, vec![inbox_url], context).await?;
    }
    Ok(())
}

#[derive(Clone)]
pub struct VerifyUrlData(pub IbisConfig);

#[async_trait]
impl UrlVerifier for VerifyUrlData {
    /// Check domain against allowlist and blocklist from config file.
    async fn verify(&self, url: &Url) -> Result<(), ActivityPubError> {
        let domain = url.domain().expect("url has domain");
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

pub(crate) fn generate_activity_id(context: &Data<IbisContext>) -> BackendResult<Url> {
    let domain = &context.config.federation.domain;
    let id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    Ok(Url::parse(&format!(
        "{}://{}/activity/{}",
        http_protocol_str(),
        domain,
        id
    ))?)
}
