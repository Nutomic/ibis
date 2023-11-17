extern crate fediwiki;

mod common;

use crate::common::{get_query, post, setup, CLIENT};
use common::get;
use fediwiki::api::{CreateArticle, FollowInstance, GetArticle, ResolveObject};
use fediwiki::error::MyResult;
use fediwiki::federation::objects::article::DbArticle;
use fediwiki::federation::objects::instance::DbInstance;
use fediwiki::start;
use serial_test::serial;
use url::Url;

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
        get_query::<DbArticle, _>(hostname, &"article".to_string(), Some(get_article.clone()))
            .await;
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
        get_query(hostname, &"article".to_string(), Some(get_article.clone())).await?;
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

#[tokio::test]
#[serial]
async fn test_synchronize_articles() -> MyResult<()> {
    setup();
    let hostname_alpha = "localhost:8131";
    let hostname_beta = "localhost:8132";
    let handle_alpha = tokio::task::spawn(async {
        start(hostname_alpha).await.unwrap();
    });
    let handle_beta = tokio::task::spawn(async {
        start(hostname_beta).await.unwrap();
    });

    // create article on alpha
    let create_article = CreateArticle {
        title: "Manu_Chao".to_string(),
        text: "Lorem ipsum".to_string(),
    };
    let create_res: DbArticle = post(hostname_alpha, "article", &create_article).await?;
    assert_eq!(create_article.title, create_res.title);
    assert!(create_res.local);

    // article is not yet on beta
    let get_article = GetArticle {
        title: "Manu_Chao".to_string(),
    };
    let get_res = get_query::<DbArticle, _>(
        hostname_beta,
        &"article".to_string(),
        Some(get_article.clone()),
    )
    .await;
    assert!(get_res.is_err());

    // fetch alpha instance on beta, articles are also fetched automatically
    let resolve_object = ResolveObject {
        id: Url::parse(&format!("http://{hostname_alpha}"))?,
    };
    get_query::<DbInstance, _>(hostname_beta, "resolve_object", Some(resolve_object)).await?;

    // get the article and compare
    let get_res: DbArticle = get_query(
        hostname_beta,
        &"article".to_string(),
        Some(get_article.clone()),
    )
    .await?;
    assert_eq!(create_res.ap_id, get_res.ap_id);
    assert_eq!(create_article.title, get_res.title);
    assert_eq!(create_article.text, get_res.text);
    assert!(!get_res.local);

    handle_alpha.abort();
    handle_beta.abort();
    Ok(())
}
