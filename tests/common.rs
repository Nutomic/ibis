use fediwiki::api::{FollowInstance, ResolveObject};
use fediwiki::error::MyResult;
use fediwiki::federation::objects::instance::DbInstance;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::de::Deserialize;
use serde::ser::Serialize;
use std::sync::Once;
use tokio::task::JoinHandle;
use tracing::log::LevelFilter;
use url::Url;
use fediwiki::start;

pub static CLIENT: Lazy<Client> = Lazy::new(Client::new);

pub struct TestData {
    pub hostname_alpha: &'static str,
    pub hostname_beta:&'static str,
    handle_alpha: JoinHandle<()>,
    handle_beta: JoinHandle<()>,
}

impl TestData {
    pub fn start() -> Self {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            env_logger::builder()
                .filter_level(LevelFilter::Warn)
                .filter_module("activitypub_federation", LevelFilter::Info)
                .filter_module("fediwiki", LevelFilter::Info)
                .init();
        });

        let hostname_alpha = "localhost:8131";
        let hostname_beta = "localhost:8132";
        let handle_alpha = tokio::task::spawn(async {
            start(hostname_alpha).await.unwrap();
        });
        let handle_beta = tokio::task::spawn(async {
            start(hostname_beta).await.unwrap();
        });
        Self {
            hostname_alpha,
            hostname_beta,
            handle_alpha,
            handle_beta,
        }
    }

    pub fn stop(self) -> MyResult<()>{
        self.handle_alpha.abort();
        self.handle_beta.abort();
        Ok(())
    }
}

pub async fn get<T>(hostname: &str, endpoint: &str) -> MyResult<T>
where
    T: for<'de> Deserialize<'de>,
{
    get_query(hostname, endpoint, None::<i32>).await
}

pub async fn get_query<T, R>(hostname: &str, endpoint: &str, query: Option<R>) -> MyResult<T>
where
    T: for<'de> Deserialize<'de>,
    R: Serialize,
{
    let mut res = CLIENT.get(format!("http://{}/api/v1/{}", hostname, endpoint));
    if let Some(query) = query {
        res = res.query(&query);
    }
    let alpha_instance: T = res.send().await?.json().await?;
    Ok(alpha_instance)
}

pub async fn post<T: Serialize, R>(hostname: &str, endpoint: &str, form: &T) -> MyResult<R>
where
    R: for<'de> Deserialize<'de>,
{
    Ok(CLIENT
        .post(format!("http://{}/api/v1/{}", hostname, endpoint))
        .form(form)
        .send()
        .await?
        .json()
        .await?)
}

pub async fn patch<T: Serialize, R>(hostname: &str, endpoint: &str, form: &T) -> MyResult<R>
where
    R: for<'de> Deserialize<'de>,
{
    Ok(CLIENT
        .patch(format!("http://{}/api/v1/{}", hostname, endpoint))
        .form(form)
        .send()
        .await?
        .json()
        .await?)
}

pub async fn follow_instance(follow_instance: &str, followed_instance: &str) -> MyResult<()> {
    // fetch beta instance on alpha
    let resolve_form = ResolveObject {
        id: Url::parse(&format!("http://{}", followed_instance))?,
    };
    let beta_instance_resolved: DbInstance =
        get_query(followed_instance, "resolve_object", Some(resolve_form)).await?;

    // send follow
    let follow_form = FollowInstance {
        instance_id: beta_instance_resolved.ap_id,
    };
    // cant use post helper because follow doesnt return json
    CLIENT
        .post(format!("http://{}/api/v1/instance/follow", follow_instance))
        .form(&follow_form)
        .send()
        .await?;
    Ok(())
}
