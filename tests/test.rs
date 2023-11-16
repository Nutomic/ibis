extern crate fediwiki;

use fediwiki::api::{CreateArticle, FollowInstance, GetArticle, ResolveObject};
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
async fn test_create_and_read_article() -> MyResult<()> {
    setup();
    let hostname = "localhost:8131";
    let handle = tokio::task::spawn(async {
        start(hostname).await.unwrap();
    });

    // error on nonexistent article
    let get_article = GetArticle {
        title: "Manu_Chao".to_string(),
    };
    let not_found =
        get_query::<DbArticle, _>(hostname, &format!("article"), Some(get_article.clone())).await;
    assert!(not_found.is_err());

    // create article
    let create_article = CreateArticle {
        title: get_article.title.to_string(),
        text: "Lorem ipsum".to_string(),
    };
    let create_res: DbArticle = post(hostname, "article", &create_article).await?;
    assert_eq!(create_article.title, create_res.title);
    assert!(create_res.local);

    // now article can be read
    let get_res: DbArticle =
        get_query(hostname, &format!("article"), Some(get_article.clone())).await?;
    assert_eq!(create_article.title, get_res.title);
    assert_eq!(create_article.text, get_res.text);
    assert!(get_res.local);

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
    // cant use post helper because follow doesnt return json
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

async fn post<T: Serialize, R>(hostname: &str, endpoint: &str, form: &T) -> MyResult<R>
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
