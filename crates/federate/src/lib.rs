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
    impls::{
        IbisContext,
        sent_activity::{SentActivity, SentActivityInsertForm},
    },
};
use log::{info, warn};
use objects::{instance::InstanceWrapper, user::PersonWrapper};
use rand::{Rng, distr::Alphanumeric, rng};
use routes::AnnouncableActivities;
use serde::Serialize;
use std::fmt::Debug;
use url::Url;

pub mod activities;
pub mod collections;
pub mod nodeinfo;
pub mod objects;
pub mod routes;
pub mod validate;
pub mod webfinger;

pub async fn send_ibis_activity<Activity, ActorType>(
    actor: &ActorType,
    activity: Activity,
    recipients: Vec<Url>,
    context: &Data<IbisContext>,
) -> BackendResult<()>
where
    Activity: ActivityHandler + Serialize + Debug + Send + Sync + 'static,
    ActorType: Actor + Sync + Clone,
    <Activity as ActivityHandler>::Error: From<activitypub_federation::error::Error>,
{
    let form = SentActivityInsertForm {
        id: activity.id().clone().into(),
        json: serde_json::to_string(&activity)?,
    };
    SentActivity::create(form, context)?;
    info!("Sending activity {}", activity.id());

    let actor = actor.clone();
    let context = context.reset_request_count();
    let join = tokio::spawn(async move {
        let activity = WithContext::new_default(activity);
        queue_activity(&activity, &actor, recipients, &context)
            .await
            .inspect_err(|e| warn!("Failed to send activity: {e}"))
            .ok();
    });

    // In production do activity send in background to avoid slow api calls. For tests use
    // synchronous federation.
    if cfg!(debug_assertions) {
        join.await?;
    }
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
        send_ibis_activity(actor, activity, vec![inbox_url], context).await?;
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
    let domain = &context.conf.federation.domain;
    let id: String = rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();
    Ok(Url::parse(&format!(
        "{}://{}/activity/{}",
        http_protocol_str(),
        domain,
        id
    ))?)
}
