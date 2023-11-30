extern crate fediwiki;

mod common;

use crate::common::{
    create_article, edit_article, edit_article_with_conflict, follow_instance, get_article,
    get_query, post, TestData, TEST_ARTICLE_DEFAULT_TEXT,
};
use common::get;
use fediwiki::api::{
    ApiConflict, EditArticleData, ForkArticleData, ResolveObject, SearchArticleData,
};
use fediwiki::database::article::{ArticleView, DbArticle};
use fediwiki::error::MyResult;
use fediwiki::federation::objects::edit::ApubEdit;
use fediwiki::federation::objects::instance::DbInstance;
use serial_test::serial;
use url::Url;

#[tokio::test]
#[serial]
async fn test_create_read_and_edit_article() -> MyResult<()> {
    let data = TestData::start();

    // create article
    let title = "Manu_Chao".to_string();
    let create_res = create_article(data.hostname_alpha, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert!(create_res.article.local);

    // now article can be read
    let get_res = get_article(data.hostname_alpha, create_res.article.id).await?;
    assert_eq!(title, get_res.article.title);
    assert_eq!(TEST_ARTICLE_DEFAULT_TEXT, get_res.article.text);
    assert!(get_res.article.local);

    // error on article which wasnt federated
    let not_found = get_article(data.hostname_beta, create_res.article.id).await;
    assert!(not_found.is_err());

    // edit article
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum 2".to_string(),
        previous_version: get_res.article.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = edit_article(data.hostname_alpha, &edit_form).await?;
    assert_eq!(edit_form.new_text, edit_res.article.text);
    assert_eq!(2, edit_res.edits.len());

    let search_form = SearchArticleData {
        query: title.clone(),
    };
    let search_res: Vec<DbArticle> =
        get_query(data.hostname_alpha, "search", Some(search_form)).await?;
    assert_eq!(1, search_res.len());
    assert_eq!(edit_res.article, search_res[0]);

    data.stop()
}

#[tokio::test]
#[serial]
async fn test_create_duplicate_article() -> MyResult<()> {
    let data = TestData::start();

    // create article
    let title = "Manu_Chao".to_string();
    let create_res = create_article(data.hostname_alpha, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert!(create_res.article.local);

    let create_res = create_article(data.hostname_alpha, title.clone()).await;
    assert!(create_res.is_err());

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

    follow_instance(data.hostname_alpha, data.hostname_beta).await?;

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
    let title = "Manu_Chao".to_string();
    let create_res = create_article(data.hostname_alpha, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert_eq!(1, create_res.edits.len());
    assert!(create_res.article.local);

    // edit the article
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum 2\n".to_string(),
        previous_version: create_res.article.latest_version,
        resolve_conflict_id: None,
    };
    edit_article(data.hostname_alpha, &edit_form).await?;

    // article is not yet on beta
    let get_res = get_article(data.hostname_beta, create_res.article.id).await;
    assert!(get_res.is_err());

    // fetch alpha instance on beta, articles are also fetched automatically
    let resolve_object = ResolveObject {
        id: Url::parse(&format!("http://{}", data.hostname_alpha))?,
    };
    get_query::<DbInstance, _>(data.hostname_beta, "resolve_instance", Some(resolve_object))
        .await?;

    // get the article and compare
    let get_res = get_article(data.hostname_beta, create_res.article.id).await?;
    assert_eq!(create_res.article.ap_id, get_res.article.ap_id);
    assert_eq!(title, get_res.article.title);
    assert_eq!(2, get_res.edits.len());
    assert_eq!(edit_form.new_text, get_res.article.text);
    assert!(!get_res.article.local);

    data.stop()
}

#[tokio::test]
#[serial]
async fn test_edit_local_article() -> MyResult<()> {
    let data = TestData::start();

    follow_instance(data.hostname_alpha, data.hostname_beta).await?;

    // create new article
    let title = "Manu_Chao".to_string();
    let create_res = create_article(data.hostname_beta, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert!(create_res.article.local);

    // article should be federated to alpha
    let get_res = get_article(data.hostname_alpha, create_res.article.id).await?;
    assert_eq!(create_res.article.title, get_res.article.title);
    assert_eq!(1, get_res.edits.len());
    assert!(!get_res.article.local);
    assert_eq!(create_res.article.text, get_res.article.text);

    // edit the article
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum 2".to_string(),
        previous_version: get_res.article.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = edit_article(data.hostname_beta, &edit_form).await?;
    assert_eq!(edit_res.article.text, edit_form.new_text);
    assert_eq!(edit_res.edits.len(), 2);
    assert!(edit_res.edits[0]
        .ap_id
        .to_string()
        .starts_with(&edit_res.article.ap_id.to_string()));

    // edit should be federated to alpha
    let get_res = get_article(data.hostname_alpha, edit_res.article.id).await?;
    assert_eq!(edit_res.article.title, get_res.article.title);
    assert_eq!(edit_res.edits.len(), 2);
    assert_eq!(edit_res.article.text, get_res.article.text);

    data.stop()
}

#[tokio::test]
#[serial]
async fn test_edit_remote_article() -> MyResult<()> {
    let data = TestData::start();

    follow_instance(data.hostname_alpha, data.hostname_beta).await?;
    follow_instance(data.hostname_gamma, data.hostname_beta).await?;

    // create new article
    let title = "Manu_Chao".to_string();
    let create_res = create_article(data.hostname_beta, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert!(create_res.article.local);

    // article should be federated to alpha and gamma
    let get_res = get_article(data.hostname_alpha, create_res.article.id).await?;
    assert_eq!(create_res.article.title, get_res.article.title);
    assert_eq!(1, get_res.edits.len());
    assert!(!get_res.article.local);

    let get_res = get_article(data.hostname_gamma, create_res.article.id).await?;
    assert_eq!(create_res.article.title, get_res.article.title);
    assert_eq!(create_res.article.text, get_res.article.text);

    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum 2".to_string(),
        previous_version: get_res.article.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = edit_article(data.hostname_alpha, &edit_form).await?;
    assert_eq!(edit_form.new_text, edit_res.article.text);
    assert_eq!(2, edit_res.edits.len());
    assert!(!edit_res.article.local);
    assert!(edit_res.edits[0]
        .ap_id
        .to_string()
        .starts_with(&edit_res.article.ap_id.to_string()));

    // edit should be federated to beta and gamma
    let get_res = get_article(data.hostname_alpha, create_res.article.id).await?;
    assert_eq!(edit_res.article.title, get_res.article.title);
    assert_eq!(edit_res.edits.len(), 2);
    assert_eq!(edit_res.article.text, get_res.article.text);

    let get_res = get_article(data.hostname_gamma, create_res.article.id).await?;
    assert_eq!(edit_res.article.title, get_res.article.title);
    assert_eq!(edit_res.edits.len(), 2);
    assert_eq!(edit_res.article.text, get_res.article.text);

    data.stop()
}

#[tokio::test]
#[serial]
async fn test_local_edit_conflict() -> MyResult<()> {
    let data = TestData::start();

    // create new article
    let title = "Manu_Chao".to_string();
    let create_res = create_article(data.hostname_alpha, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert!(create_res.article.local);

    // one user edits article
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum\n".to_string(),
        previous_version: create_res.article.latest_version.clone(),
        resolve_conflict_id: None,
    };
    let edit_res = edit_article(data.hostname_alpha, &edit_form).await?;
    assert_eq!(edit_res.article.text, edit_form.new_text);
    assert_eq!(2, edit_res.edits.len());

    // another user edits article, without being aware of previous edit
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Ipsum Lorem\n".to_string(),
        previous_version: create_res.article.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = edit_article_with_conflict(data.hostname_alpha, &edit_form)
        .await?
        .unwrap();
    assert_eq!("<<<<<<< ours\nIpsum Lorem\n||||||| original\nsome\nexample\ntext\n=======\nLorem Ipsum\n>>>>>>> theirs\n", edit_res.three_way_merge);

    let conflicts: Vec<ApiConflict> =
        get_query(data.hostname_alpha, "edit_conflicts", None::<()>).await?;
    assert_eq!(1, conflicts.len());
    assert_eq!(conflicts[0], edit_res);

    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum and Ipsum Lorem\n".to_string(),
        previous_version: edit_res.previous_version,
        resolve_conflict_id: Some(edit_res.id),
    };
    let edit_res = edit_article(data.hostname_alpha, &edit_form).await?;
    assert_eq!(edit_form.new_text, edit_res.article.text);

    let conflicts: Vec<ApiConflict> =
        get_query(data.hostname_alpha, "edit_conflicts", None::<()>).await?;
    assert_eq!(0, conflicts.len());

    data.stop()
}

#[tokio::test]
#[serial]
async fn test_federated_edit_conflict() -> MyResult<()> {
    let data = TestData::start();

    follow_instance(data.hostname_alpha, data.hostname_beta).await?;

    // create new article
    let title = "Manu_Chao".to_string();
    let create_res = create_article(data.hostname_beta, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert!(create_res.article.local);

    // fetch article to gamma
    let resolve_object = ResolveObject {
        id: create_res.article.ap_id.inner().clone(),
    };
    let resolve_res: DbArticle =
        get_query(data.hostname_gamma, "resolve_article", Some(resolve_object)).await?;
    assert_eq!(create_res.article.text, resolve_res.text);

    // alpha edits article
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum\n".to_string(),
        previous_version: create_res.article.latest_version.clone(),
        resolve_conflict_id: None,
    };
    let edit_res = edit_article(data.hostname_alpha, &edit_form).await?;
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
        previous_version: create_res.article.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = edit_article(data.hostname_gamma, &edit_form).await?;
    assert_ne!(edit_form.new_text, edit_res.article.text);
    assert_eq!(2, edit_res.edits.len());
    assert!(!edit_res.article.local);

    let conflicts: Vec<ApiConflict> =
        get_query(data.hostname_gamma, "edit_conflicts", None::<()>).await?;
    assert_eq!(1, conflicts.len());

    // resolve the conflict
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "aaaa\n".to_string(),
        previous_version: conflicts[0].previous_version.clone(),
        resolve_conflict_id: Some(conflicts[0].id),
    };
    let edit_res = edit_article(data.hostname_gamma, &edit_form).await?;
    assert_eq!(edit_form.new_text, edit_res.article.text);
    assert_eq!(3, edit_res.edits.len());

    let conflicts: Vec<ApubEdit> =
        get_query(data.hostname_gamma, "edit_conflicts", None::<()>).await?;
    assert_eq!(0, conflicts.len());

    data.stop()
}

#[tokio::test]
#[serial]
async fn test_overlapping_edits_no_conflict() -> MyResult<()> {
    let data = TestData::start();

    // create new article
    let title = "Manu_Chao".to_string();
    let create_res = create_article(data.hostname_alpha, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert!(create_res.article.local);

    // one user edits article
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "my\nexample\ntext\n".to_string(),
        previous_version: create_res.article.latest_version.clone(),
        resolve_conflict_id: None,
    };
    let edit_res = edit_article(data.hostname_alpha, &edit_form).await?;
    assert_eq!(edit_res.article.text, edit_form.new_text);
    assert_eq!(2, edit_res.edits.len());

    // another user edits article, without being aware of previous edit
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "some\nexample\narticle\n".to_string(),
        previous_version: create_res.article.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = edit_article(data.hostname_alpha, &edit_form).await?;
    let conflicts: Vec<ApiConflict> =
        get_query(data.hostname_alpha, "edit_conflicts", None::<()>).await?;
    assert_eq!(0, conflicts.len());
    assert_eq!(3, edit_res.edits.len());
    assert_eq!("my\nexample\narticle\n", edit_res.article.text);

    data.stop()
}

#[tokio::test]
#[serial]
async fn test_fork_article() -> MyResult<()> {
    let data = TestData::start();

    // create article
    let title = "Manu_Chao".to_string();
    let create_res = create_article(data.hostname_alpha, title.clone()).await?;
    assert_eq!(title, create_res.article.title);
    assert!(create_res.article.local);

    // fetch on beta
    let resolve_object = ResolveObject {
        id: create_res.article.ap_id.into_inner(),
    };
    let resolve_res: ArticleView =
        get_query(data.hostname_beta, "resolve_article", Some(resolve_object)).await?;
    let resolved_article = resolve_res.article;
    assert_eq!(create_res.edits.len(), resolve_res.edits.len());

    // fork the article to local instance
    let fork_form = ForkArticleData {
        article_id: resolved_article.id,
    };
    let fork_res: ArticleView = post(data.hostname_beta, "article/fork", &fork_form).await?;
    let forked_article = fork_res.article;
    assert_eq!(resolved_article.title, forked_article.title);
    assert_eq!(resolved_article.text, forked_article.text);
    assert_eq!(resolve_res.edits, fork_res.edits);
    assert_eq!(
        resolved_article.latest_version,
        forked_article.latest_version
    );
    assert_ne!(resolved_article.ap_id, forked_article.ap_id);
    assert!(forked_article.local);

    let beta_instance: DbInstance = get(data.hostname_beta, "instance").await?;
    assert_eq!(forked_article.instance_id, beta_instance.ap_id);

    // now search returns two articles for this title (original and forked)
    let search_form = SearchArticleData {
        query: title.clone(),
    };
    let search_res: Vec<DbArticle> =
        get_query(data.hostname_beta, "search", Some(search_form)).await?;
    assert_eq!(2, search_res.len());

    data.stop()
}
