extern crate fediwiki;

use fediwiki::api::{FollowInstance, ResolveObject};
use fediwiki::error::MyResult;
use fediwiki::federation::objects::article::DbArticle;
use fediwiki::federation::objects::instance::DbInstance;
use fediwiki::start;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serial_test::serial;
use std::sync::Once;
use tracing::log::LevelFilter;
use url::Url;

fn setup() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        env_logger::builder()
            .filter_level(LevelFilter::Warn)
            .filter_module("activitypub_federation", LevelFilter::Info)
            .filter_module("fediwiki", LevelFilter::Info)
            .init();
    });
}

static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

#[tokio::test]
#[serial]
async fn test_get_article() -> MyResult<()> {
    setup();
    let hostname = "localhost:8131";
    let handle = tokio::task::spawn(async {
        start(hostname).await.unwrap();
    });

    let title = "Manu_Chao";
    let res: DbArticle = get(hostname, &format!("article/{title}")).await?;
    assert_eq!(title, res.title);
    assert!(res.local);
    handle.abort();
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_follow_instance() -> MyResult<()> {
    setup();
    let hostname_alpha = "localhost:8131";
    let hostname_beta = "localhost:8132";
    let handle_alpha = tokio::task::spawn(async {
        start(hostname_alpha).await.unwrap();
    });
    let handle_beta = tokio::task::spawn(async {
        start(hostname_beta).await.unwrap();
    });

    // check initial state
    let alpha_instance: DbInstance = get(hostname_alpha, "instance").await?;
    assert_eq!(0, alpha_instance.follows.len());
    let beta_instance: DbInstance = get(hostname_beta, "instance").await?;
    assert_eq!(0, beta_instance.followers.len());

    // fetch beta instance on alpha
    let resolve_object = ResolveObject {
        id: Url::parse(&format!("http://{hostname_beta}"))?,
    };
    let beta_instance_resolved: DbInstance =
        get_query(hostname_beta, "resolve_object", Some(resolve_object)).await?;

    // send follow
    let follow_instance = FollowInstance {
        instance_id: beta_instance_resolved.ap_id,
    };
    CLIENT
        .post(format!("http://{hostname_alpha}/api/v1/instance/follow"))
        .form(&follow_instance)
        .send()
        .await?;

    // check that follow was federated
    let beta_instance: DbInstance = get(hostname_beta, "instance").await?;
    assert_eq!(1, beta_instance.followers.len());

    let alpha_instance: DbInstance = get(hostname_alpha, "instance").await?;
    assert_eq!(1, alpha_instance.follows.len());

    handle_alpha.abort();
    handle_beta.abort();
    Ok(())
}

async fn get<T>(hostname: &str, endpoint: &str) -> MyResult<T>
where
    T: for<'de> Deserialize<'de>,
{
    get_query(hostname, endpoint, None::<i32>).await
}

async fn get_query<T, R>(hostname: &str, endpoint: &str, query: Option<R>) -> MyResult<T>
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
