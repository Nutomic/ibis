extern crate ibis;

mod common;

use crate::common::{
    create_article, edit_article, edit_article_with_conflict, follow_instance, get_article,
    get_query, TestData, CLIENT, TEST_ARTICLE_DEFAULT_TEXT,
};
use crate::common::{fork_article, handle_json_res, login};
use crate::common::{get_conflicts, register};
use common::get;
use ibis::api::article::{CreateArticleData, EditArticleData, ForkArticleData};
use ibis::api::{ResolveObject, SearchArticleData};
use ibis::database::article::{ArticleView, DbArticle};
use ibis::database::instance::{DbInstance, InstanceView};
use ibis::error::MyResult;
use pretty_assertions::{assert_eq, assert_ne};
use url::Url;

#[tokio::test]
async fn test_create_read_and_edit_article() -> MyResult<()> {
    let data = TestData::start().await;

    // create article
    let title = "Manu_Chao".to_string();
    let create_res = create_article(&data.alpha, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert!(create_res.article.local);

    // now article can be read
    let get_res = get_article(&data.alpha.hostname, create_res.article.id).await?;
    assert_eq!(title, get_res.article.title);
    assert_eq!(TEST_ARTICLE_DEFAULT_TEXT, get_res.article.text);
    assert!(get_res.article.local);

    // error on article which wasnt federated
    let not_found = get_article(&data.beta.hostname, create_res.article.id).await;
    assert!(not_found.is_err());

    // edit article
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum 2".to_string(),
        previous_version_id: get_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = edit_article(&data.alpha, &edit_form).await?;
    assert_eq!(edit_form.new_text, edit_res.article.text);
    assert_eq!(2, edit_res.edits.len());

    let search_form = SearchArticleData {
        query: title.clone(),
    };
    let search_res: Vec<DbArticle> =
        get_query(&data.alpha.hostname, "search", Some(search_form)).await?;
    assert_eq!(1, search_res.len());
    assert_eq!(edit_res.article, search_res[0]);

    data.stop()
}

#[tokio::test]
async fn test_create_duplicate_article() -> MyResult<()> {
    let data = TestData::start().await;

    // create article
    let title = "Manu_Chao".to_string();
    let create_res = create_article(&data.alpha, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert!(create_res.article.local);

    let create_res = create_article(&data.alpha, title.clone()).await;
    assert!(create_res.is_err());

    data.stop()
}

#[tokio::test]
async fn test_follow_instance() -> MyResult<()> {
    let data = TestData::start().await;

    // check initial state
    let alpha_instance: InstanceView = get(&data.alpha.hostname, "instance").await?;
    assert_eq!(0, alpha_instance.followers.len());
    assert_eq!(0, alpha_instance.following.len());
    let beta_instance: InstanceView = get(&data.beta.hostname, "instance").await?;
    assert_eq!(0, beta_instance.followers.len());
    assert_eq!(0, beta_instance.following.len());

    follow_instance(&data.alpha, &data.beta.hostname).await?;

    // check that follow was federated
    let alpha_instance: InstanceView = get(&data.alpha.hostname, "instance").await?;
    assert_eq!(1, alpha_instance.following.len());
    assert_eq!(0, alpha_instance.followers.len());
    assert_eq!(
        beta_instance.instance.ap_id,
        alpha_instance.following[0].ap_id
    );

    let beta_instance: InstanceView = get(&data.beta.hostname, "instance").await?;
    assert_eq!(0, beta_instance.following.len());
    assert_eq!(1, beta_instance.followers.len());
    // TODO: compare full ap_id of alpha user, but its not available through api yet
    assert_eq!(
        alpha_instance.instance.ap_id.inner().domain(),
        beta_instance.followers[0].ap_id.inner().domain()
    );

    data.stop()
}

#[tokio::test]
async fn test_synchronize_articles() -> MyResult<()> {
    let data = TestData::start().await;

    // create article on alpha
    let title = "Manu_Chao".to_string();
    let create_res = create_article(&data.alpha, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert_eq!(1, create_res.edits.len());
    assert!(create_res.article.local);

    // edit the article
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum 2\n".to_string(),
        previous_version_id: create_res.latest_version,
        resolve_conflict_id: None,
    };
    edit_article(&data.alpha, &edit_form).await?;

    // article is not yet on beta
    let get_res = get_article(&data.beta.hostname, create_res.article.id).await;
    assert!(get_res.is_err());

    // fetch alpha instance on beta, articles are also fetched automatically
    let resolve_object = ResolveObject {
        id: Url::parse(&format!("http://{}", &data.alpha.hostname))?,
    };
    get_query::<DbInstance, _>(
        &data.beta.hostname,
        "resolve_instance",
        Some(resolve_object),
    )
    .await?;

    // get the article and compare
    let get_res = get_article(&data.beta.hostname, create_res.article.id).await?;
    assert_eq!(create_res.article.ap_id, get_res.article.ap_id);
    assert_eq!(title, get_res.article.title);
    assert_eq!(2, get_res.edits.len());
    assert_eq!(edit_form.new_text, get_res.article.text);
    assert!(!get_res.article.local);

    data.stop()
}

#[tokio::test]
async fn test_edit_local_article() -> MyResult<()> {
    let data = TestData::start().await;

    follow_instance(&data.alpha, &data.beta.hostname).await?;

    // create new article
    let title = "Manu_Chao".to_string();
    let create_res = create_article(&data.beta, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert!(create_res.article.local);

    // article should be federated to alpha
    let get_res = get_article(&data.alpha.hostname, create_res.article.id).await?;
    assert_eq!(create_res.article.title, get_res.article.title);
    assert_eq!(1, get_res.edits.len());
    assert!(!get_res.article.local);
    assert_eq!(create_res.article.text, get_res.article.text);

    // edit the article
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum 2".to_string(),
        previous_version_id: get_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = edit_article(&data.beta, &edit_form).await?;
    assert_eq!(edit_res.article.text, edit_form.new_text);
    assert_eq!(edit_res.edits.len(), 2);
    assert!(edit_res.edits[0]
        .ap_id
        .to_string()
        .starts_with(&edit_res.article.ap_id.to_string()));

    // edit should be federated to alpha
    let get_res = get_article(&data.alpha.hostname, edit_res.article.id).await?;
    assert_eq!(edit_res.article.title, get_res.article.title);
    assert_eq!(edit_res.edits.len(), 2);
    assert_eq!(edit_res.article.text, get_res.article.text);

    data.stop()
}

#[tokio::test]
async fn test_edit_remote_article() -> MyResult<()> {
    let data = TestData::start().await;

    follow_instance(&data.alpha, &data.beta.hostname).await?;
    follow_instance(&data.gamma, &data.beta.hostname).await?;

    // create new article
    let title = "Manu_Chao".to_string();
    let create_res = create_article(&data.beta, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert!(create_res.article.local);

    // article should be federated to alpha and gamma
    let get_res = get_article(&data.alpha.hostname, create_res.article.id).await?;
    assert_eq!(create_res.article.title, get_res.article.title);
    assert_eq!(1, get_res.edits.len());
    assert!(!get_res.article.local);

    let get_res = get_article(&data.gamma.hostname, create_res.article.id).await?;
    assert_eq!(create_res.article.title, get_res.article.title);
    assert_eq!(create_res.article.text, get_res.article.text);

    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum 2".to_string(),
        previous_version_id: get_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = edit_article(&data.alpha, &edit_form).await?;
    assert_eq!(edit_form.new_text, edit_res.article.text);
    assert_eq!(2, edit_res.edits.len());
    assert!(!edit_res.article.local);
    assert!(edit_res.edits[0]
        .ap_id
        .to_string()
        .starts_with(&edit_res.article.ap_id.to_string()));

    // edit should be federated to beta and gamma
    let get_res = get_article(&data.alpha.hostname, create_res.article.id).await?;
    assert_eq!(edit_res.article.title, get_res.article.title);
    assert_eq!(edit_res.edits.len(), 2);
    assert_eq!(edit_res.article.text, get_res.article.text);

    let get_res = get_article(&data.gamma.hostname, create_res.article.id).await?;
    assert_eq!(edit_res.article.title, get_res.article.title);
    assert_eq!(edit_res.edits.len(), 2);
    assert_eq!(edit_res.article.text, get_res.article.text);

    data.stop()
}

#[tokio::test]
async fn test_local_edit_conflict() -> MyResult<()> {
    let data = TestData::start().await;

    // create new article
    let title = "Manu_Chao".to_string();
    let create_res = create_article(&data.alpha, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert!(create_res.article.local);

    // one user edits article
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum\n".to_string(),
        previous_version_id: create_res.latest_version.clone(),
        resolve_conflict_id: None,
    };
    let edit_res = edit_article(&data.alpha, &edit_form).await?;
    assert_eq!(edit_res.article.text, edit_form.new_text);
    assert_eq!(2, edit_res.edits.len());

    // another user edits article, without being aware of previous edit
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Ipsum Lorem\n".to_string(),
        previous_version_id: create_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = edit_article_with_conflict(&data.alpha, &edit_form)
        .await?
        .unwrap();
    assert_eq!("<<<<<<< ours\nIpsum Lorem\n||||||| original\nsome\nexample\ntext\n=======\nLorem Ipsum\n>>>>>>> theirs\n", edit_res.three_way_merge);

    let conflicts = get_conflicts(&data.alpha).await?;
    assert_eq!(1, conflicts.len());
    assert_eq!(conflicts[0], edit_res);

    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum and Ipsum Lorem\n".to_string(),
        previous_version_id: edit_res.previous_version_id,
        resolve_conflict_id: Some(edit_res.id),
    };
    let edit_res = edit_article(&data.alpha, &edit_form).await?;
    assert_eq!(edit_form.new_text, edit_res.article.text);

    let conflicts = get_conflicts(&data.alpha).await?;
    assert_eq!(0, conflicts.len());

    data.stop()
}

#[tokio::test]
async fn test_federated_edit_conflict() -> MyResult<()> {
    let data = TestData::start().await;

    follow_instance(&data.alpha, &data.beta.hostname).await?;

    // create new article
    let title = "Manu_Chao".to_string();
    let create_res = create_article(&data.beta, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert!(create_res.article.local);

    // fetch article to gamma
    let resolve_object = ResolveObject {
        id: create_res.article.ap_id.inner().clone(),
    };
    let resolve_res: ArticleView = get_query(
        &data.gamma.hostname,
        "resolve_article",
        Some(resolve_object),
    )
    .await?;
    assert_eq!(create_res.article.text, resolve_res.article.text);

    // alpha edits article
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum\n".to_string(),
        previous_version_id: create_res.latest_version.clone(),
        resolve_conflict_id: None,
    };
    let edit_res = edit_article(&data.alpha, &edit_form).await?;
    assert_eq!(edit_res.article.text, edit_form.new_text);
    assert_eq!(2, edit_res.edits.len());
    assert!(!edit_res.article.local);
    assert!(edit_res.edits[1]
        .ap_id
        .to_string()
        .starts_with(&edit_res.article.ap_id.to_string()));

    // gamma also edits, as its not the latest version there is a conflict. local version should
    // not be updated with this conflicting version, instead user needs to handle the conflict
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "aaaa\n".to_string(),
        previous_version_id: create_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = edit_article(&data.gamma, &edit_form).await?;
    assert_ne!(edit_form.new_text, edit_res.article.text);
    assert_eq!(1, edit_res.edits.len());
    assert!(!edit_res.article.local);

    let conflicts = get_conflicts(&data.gamma).await?;
    assert_eq!(1, conflicts.len());

    // resolve the conflict
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "aaaa\n".to_string(),
        previous_version_id: conflicts[0].previous_version_id.clone(),
        resolve_conflict_id: Some(conflicts[0].id.clone()),
    };
    let edit_res = edit_article(&data.gamma, &edit_form).await?;
    assert_eq!(edit_form.new_text, edit_res.article.text);
    assert_eq!(3, edit_res.edits.len());

    let conflicts = get_conflicts(&data.gamma).await?;
    assert_eq!(0, conflicts.len());

    data.stop()
}

#[tokio::test]
async fn test_overlapping_edits_no_conflict() -> MyResult<()> {
    let data = TestData::start().await;

    // create new article
    let title = "Manu_Chao".to_string();
    let create_res = create_article(&data.alpha, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert!(create_res.article.local);

    // one user edits article
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "my\nexample\ntext\n".to_string(),
        previous_version_id: create_res.latest_version.clone(),
        resolve_conflict_id: None,
    };
    let edit_res = edit_article(&data.alpha, &edit_form).await?;
    assert_eq!(edit_res.article.text, edit_form.new_text);
    assert_eq!(2, edit_res.edits.len());

    // another user edits article, without being aware of previous edit
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "some\nexample\narticle\n".to_string(),
        previous_version_id: create_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = edit_article(&data.alpha, &edit_form).await?;
    let conflicts = get_conflicts(&data.alpha).await?;
    assert_eq!(0, conflicts.len());
    assert_eq!(3, edit_res.edits.len());
    assert_eq!("my\nexample\narticle\n", edit_res.article.text);

    data.stop()
}

#[tokio::test]
async fn test_fork_article() -> MyResult<()> {
    let data = TestData::start().await;

    // create article
    let title = "Manu_Chao".to_string();
    let create_res = create_article(&data.alpha, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert!(create_res.article.local);

    // fetch on beta
    let resolve_object = ResolveObject {
        id: create_res.article.ap_id.into_inner(),
    };
    let resolve_res: ArticleView =
        get_query(&data.beta.hostname, "resolve_article", Some(resolve_object)).await?;
    let resolved_article = resolve_res.article;
    assert_eq!(create_res.edits.len(), resolve_res.edits.len());

    // fork the article to local instance
    let fork_form = ForkArticleData {
        article_id: resolved_article.id,
    };
    let fork_res = fork_article(&data.beta, &fork_form).await?;
    let forked_article = fork_res.article;
    assert_eq!(resolved_article.title, forked_article.title);
    assert_eq!(resolved_article.text, forked_article.text);
    assert_eq!(resolve_res.edits.len(), fork_res.edits.len());
    assert_eq!(resolve_res.edits[0].diff, fork_res.edits[0].diff);
    assert_eq!(resolve_res.edits[0].hash, fork_res.edits[0].hash);
    assert_ne!(resolve_res.edits[0].id, fork_res.edits[0].id);
    assert_eq!(resolve_res.latest_version, fork_res.latest_version);
    assert_ne!(resolved_article.ap_id, forked_article.ap_id);
    assert!(forked_article.local);

    let beta_instance: InstanceView = get(&data.beta.hostname, "instance").await?;
    assert_eq!(forked_article.instance_id, beta_instance.instance.id);

    // now search returns two articles for this title (original and forked)
    let search_form = SearchArticleData {
        query: title.clone(),
    };
    let search_res: Vec<DbArticle> =
        get_query(&data.beta.hostname, "search", Some(search_form)).await?;
    assert_eq!(2, search_res.len());

    data.stop()
}

#[tokio::test]
async fn test_user_registration_login() -> MyResult<()> {
    let data = TestData::start().await;
    let username = "my_user";
    let password = "hunter2";
    let register = register(&data.alpha.hostname, username, password).await?;
    assert!(!register.jwt.is_empty());

    let invalid_login = login(&data.alpha, username, "asd123").await;
    assert!(invalid_login.is_err());

    let valid_login = login(&data.alpha, username, password).await?;
    assert!(!valid_login.jwt.is_empty());

    let title = "Manu_Chao".to_string();
    let create_form = CreateArticleData {
        title: title.clone(),
    };

    let req = CLIENT
        .post(format!("http://{}/api/v1/article", &data.alpha.hostname))
        .form(&create_form)
        .bearer_auth(valid_login.jwt);
    let create_res: ArticleView = handle_json_res(req).await?;
    assert_eq!(title, create_res.article.title);

    data.stop()
}
