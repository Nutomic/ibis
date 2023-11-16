extern crate fediwiki;

use fediwiki::api::{FollowInstance, ResolveObject};
use fediwiki::error::MyResult;
use fediwiki::federation::objects::article::DbArticle;
use fediwiki::federation::objects::instance::DbInstance;
use fediwiki::start;
use once_cell::sync::Lazy;
use reqwest::Client;
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
    let res: DbArticle = reqwest::get(format!("http://{hostname}/api/v1/article/{title}"))
        .await?
        .json()
        .await?;
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
    let alpha_instance: DbInstance = CLIENT
        .get(format!("http://{hostname_alpha}/api/v1/instance"))
        .send()
        .await?
        .json()
        .await?;
    assert_eq!(0, alpha_instance.follows.len());
    let beta_instance: DbInstance = CLIENT
        .get(format!("http://{hostname_beta}/api/v1/instance"))
        .send()
        .await?
        .json()
        .await?;
    assert_eq!(0, beta_instance.followers.len());

    // fetch beta instance on alpha
    let resolve_object = ResolveObject {
        id: Url::parse(&format!("http://{hostname_beta}"))?,
    };
    let beta_instance_resolved: DbInstance = CLIENT
        .get(format!("http://{hostname_alpha}/api/v1/resolve_object"))
        .query(&resolve_object)
        .send()
        .await?
        .json()
        .await?;

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
    let beta_instance: DbInstance = CLIENT
        .get(format!("http://{hostname_beta}/api/v1/instance"))
        .send()
        .await?
        .json()
        .await?;
    assert_eq!(1, beta_instance.followers.len());

    let alpha_instance: DbInstance = CLIENT
        .get(format!("http://{hostname_alpha}/api/v1/instance"))
        .send()
        .await?
        .json()
        .await?;
    assert_eq!(1, alpha_instance.follows.len());

    handle_alpha.abort();
    handle_beta.abort();
    Ok(())
}
