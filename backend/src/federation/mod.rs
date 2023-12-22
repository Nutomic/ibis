use crate::database::MyDataHandle;
use activitypub_federation::activity_sending::SendActivityTask;
use activitypub_federation::config::Data;
use activitypub_federation::protocol::context::WithContext;
use activitypub_federation::traits::{ActivityHandler, Actor};
use serde::Serialize;
use std::fmt::Debug;
use tracing::log::warn;
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
