extern crate fediwiki;

mod common;

use crate::common::{follow_instance, get_query, patch, post, TestData};
use common::get;
use fediwiki::api::{CreateArticle, EditArticle, GetArticle, ResolveObject};
use fediwiki::error::MyResult;
use fediwiki::federation::objects::article::DbArticle;
use fediwiki::federation::objects::instance::DbInstance;
use serial_test::serial;
use url::Url;

#[tokio::test]
#[serial]
async fn test_create_and_read_article() -> MyResult<()> {
    let data = TestData::start();

    // error on nonexistent article
    let get_article = GetArticle {
        title: "Manu_Chao".to_string(),
    };
    let not_found = get_query::<DbArticle, _>(
        data.hostname_alpha,
        &"article".to_string(),
        Some(get_article.clone()),
    )
    .await;
    assert!(not_found.is_err());

    // create article
    let create_article = CreateArticle {
        title: get_article.title.to_string(),
        text: "Lorem ipsum".to_string(),
    };
    let create_res: DbArticle = post(data.hostname_alpha, "article", &create_article).await?;
    assert_eq!(create_article.title, create_res.title);
    assert!(create_res.local);

    // now article can be read
    let get_res: DbArticle = get_query(
        data.hostname_alpha,
        &"article".to_string(),
        Some(get_article.clone()),
    )
    .await?;
    assert_eq!(create_article.title, get_res.title);
    assert_eq!(create_article.text, get_res.text);
    assert!(get_res.local);

    data.stop()
}

#[tokio::test]
#[serial]
async fn test_follow_instance() -> MyResult<()> {
    let data = TestData::start();

    // check initial state
    let alpha_instance: DbInstance = get(data.hostname_alpha, "instance").await?;
    assert_eq!(0, alpha_instance.follows.len());
    let beta_instance: DbInstance = get(data.hostname_beta, "instance").await?;
    assert_eq!(0, beta_instance.followers.len());

    follow_instance(data.hostname_alpha, &data.hostname_beta).await?;

    // check that follow was federated
    let beta_instance: DbInstance = get(data.hostname_beta, "instance").await?;
    assert_eq!(1, beta_instance.followers.len());

    let alpha_instance: DbInstance = get(data.hostname_alpha, "instance").await?;
    assert_eq!(1, alpha_instance.follows.len());

    data.stop()
}

#[tokio::test]
#[serial]
async fn test_synchronize_articles() -> MyResult<()> {
    let data = TestData::start();

    // create article on alpha
    let create_article = CreateArticle {
        title: "Manu_Chao".to_string(),
        text: "Lorem ipsum".to_string(),
    };
    let create_res: DbArticle = post(data.hostname_alpha, "article", &create_article).await?;
    assert_eq!(create_article.title, create_res.title);
    assert!(create_res.local);

    // article is not yet on beta
    let get_article = GetArticle {
        title: "Manu_Chao".to_string(),
    };
    let get_res = get_query::<DbArticle, _>(
        data.hostname_beta,
        &"article".to_string(),
        Some(get_article.clone()),
    )
    .await;
    assert!(get_res.is_err());

    // fetch alpha instance on beta, articles are also fetched automatically
    let resolve_object = ResolveObject {
        id: Url::parse(&format!("http://{}", data.hostname_alpha))?,
    };
    get_query::<DbInstance, _>(data.hostname_beta, "resolve_object", Some(resolve_object)).await?;

    // get the article and compare
    let get_res: DbArticle = get_query(
        data.hostname_beta,
        &"article".to_string(),
        Some(get_article.clone()),
    )
    .await?;
    assert_eq!(create_res.ap_id, get_res.ap_id);
    assert_eq!(create_article.title, get_res.title);
    assert_eq!(create_article.text, get_res.text);
    assert!(!get_res.local);

    data.stop()
}

#[tokio::test]
#[serial]
async fn test_federate_article_changes() -> MyResult<()> {
    let data = TestData::start();

    follow_instance(data.hostname_alpha, data.hostname_beta).await?;

    // create new article
    let create_form = CreateArticle {
        title: "Manu_Chao".to_string(),
        text: "Lorem ipsum".to_string(),
    };
    let create_res: DbArticle = post(data.hostname_beta, "article", &create_form).await?;
    assert_eq!(create_res.title, create_form.title);

    // article should be federated to alpha
    let get_article = GetArticle {
        title: create_res.title.clone(),
    };
    let get_res =
        get_query::<DbArticle, _>(data.hostname_alpha, "article", Some(get_article.clone()))
            .await?;
    assert_eq!(create_res.title, get_res.title);
    assert_eq!(create_res.text, get_res.text);

    // edit the article
    let edit_form = EditArticle {
        ap_id: create_res.ap_id,
        new_text: "Lorem Ipsum 2".to_string(),
    };
    let edit_res: DbArticle = patch(data.hostname_beta, "article", &edit_form).await?;
    assert_eq!(edit_res.text, edit_form.new_text);
    assert_eq!(edit_res.edits.len(), 1);
    assert!(edit_res.edits[0].id.to_string().starts_with(&edit_res.ap_id.to_string()));

    // edit should be federated to alpha
    let get_article = GetArticle {
        title: edit_res.title.clone(),
    };
    let get_res =
        get_query::<DbArticle, _>(data.hostname_alpha, "article", Some(get_article.clone()))
            .await?;
    assert_eq!(edit_res.title, get_res.title);
    assert_eq!(edit_res.text, get_res.text);

    data.stop()
}
