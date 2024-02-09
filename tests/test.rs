extern crate ibis_lib;

mod common;

use crate::common::{TestData, TEST_ARTICLE_DEFAULT_TEXT};
use ibis_lib::common::{
    ArticleView, EditArticleData, ForkArticleData, GetArticleData, ListArticlesData,
};
use ibis_lib::common::{CreateArticleData, SearchArticleData};
use ibis_lib::common::{LoginUserData, RegisterUserData};
use ibis_lib::frontend::error::MyResult;
use pretty_assertions::{assert_eq, assert_ne};
use url::Url;

#[tokio::test]
async fn test_create_read_and_edit_local_article() -> MyResult<()> {
    let data = TestData::start().await;

    // create article
    let create_form = CreateArticleData {
        title: "Manu_Chao".to_string(),
        text: TEST_ARTICLE_DEFAULT_TEXT.to_string(),
        summary: "create article".to_string(),
    };
    let create_res = data.alpha.create_article(&create_form).await?;
    assert_eq!(create_form.title, create_res.article.title);
    assert!(create_res.article.local);

    // now article can be read
    let get_article_data = GetArticleData {
        title: Some(create_res.article.title.clone()),
        instance_id: None,
        id: None,
    };
    let get_res = data.alpha.get_article(get_article_data.clone()).await?;
    assert_eq!(create_form.title, get_res.article.title);
    assert_eq!(TEST_ARTICLE_DEFAULT_TEXT, get_res.article.text);
    assert!(get_res.article.local);

    // error on article which wasnt federated
    let not_found = data.beta.get_article(get_article_data.clone()).await;
    assert!(not_found.is_err());

    // edit article
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum 2\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: get_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = data.alpha.edit_article(&edit_form).await?;
    assert_eq!(edit_form.new_text, edit_res.article.text);
    assert_eq!(2, edit_res.edits.len());
    assert_eq!(edit_form.summary, edit_res.edits[1].summary);

    let search_form = SearchArticleData {
        query: create_form.title.clone(),
    };
    let search_res = data.alpha.search(&search_form).await?;
    assert_eq!(1, search_res.len());
    assert_eq!(edit_res.article, search_res[0]);

    let list_articles = data
        .alpha
        .list_articles(ListArticlesData {
            only_local: Some(false),
        })
        .await?;
    assert_eq!(2, list_articles.len());
    assert_eq!(edit_res.article, list_articles[0]);

    data.stop()
}

#[tokio::test]
async fn test_create_duplicate_article() -> MyResult<()> {
    let data = TestData::start().await;

    // create article
    let create_form = CreateArticleData {
        title: "Manu_Chao".to_string(),
        text: TEST_ARTICLE_DEFAULT_TEXT.to_string(),
        summary: "create article".to_string(),
    };
    let create_res = data.alpha.create_article(&create_form).await?;
    assert_eq!(create_form.title, create_res.article.title);
    assert!(create_res.article.local);

    let create_res = data.alpha.create_article(&create_form).await;
    assert!(create_res.is_err());

    data.stop()
}

#[tokio::test]
async fn test_follow_instance() -> MyResult<()> {
    let data = TestData::start().await;

    // check initial state
    let alpha_user = data.alpha.my_profile().await?;
    assert_eq!(0, alpha_user.following.len());
    let beta_instance = data.beta.get_local_instance().await?;
    assert_eq!(0, beta_instance.followers.len());

    data.alpha.follow_instance(&data.beta.hostname).await?;

    // check that follow was federated
    let alpha_user = data.alpha.my_profile().await?;
    assert_eq!(1, alpha_user.following.len());
    assert_eq!(beta_instance.instance.ap_id, alpha_user.following[0].ap_id);

    let beta_instance = data.beta.get_local_instance().await?;
    assert_eq!(1, beta_instance.followers.len());
    assert_eq!(alpha_user.person.ap_id, beta_instance.followers[0].ap_id);

    data.stop()
}

#[tokio::test]
async fn test_synchronize_articles() -> MyResult<()> {
    let data = TestData::start().await;

    // create article on alpha
    let create_form = CreateArticleData {
        title: "Manu_Chao".to_string(),
        text: TEST_ARTICLE_DEFAULT_TEXT.to_string(),
        summary: "create article".to_string(),
    };
    let create_res = data.alpha.create_article(&create_form).await?;
    assert_eq!(create_form.title, create_res.article.title);
    assert_eq!(1, create_res.edits.len());
    assert!(create_res.article.local);

    // edit the article
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum 2\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: create_res.latest_version,
        resolve_conflict_id: None,
    };
    data.alpha.edit_article(&edit_form).await?;

    // fetch alpha instance on beta, articles are also fetched automatically
    let instance = data
        .beta
        .resolve_instance(Url::parse(&format!("http://{}", &data.alpha.hostname))?)
        .await?;

    let mut get_article_data = GetArticleData {
        title: Some(create_res.article.title),
        instance_id: None,
        id: None,
    };

    // try to read remote article by name, fails without domain
    let get_res = data.beta.get_article(get_article_data.clone()).await;
    assert!(get_res.is_err());

    // get the article with instance id and compare
    get_article_data.instance_id = Some(instance.id);
    let get_res = data.beta.get_article(get_article_data).await?;
    assert_eq!(create_res.article.ap_id, get_res.article.ap_id);
    assert_eq!(create_form.title, get_res.article.title);
    assert_eq!(2, get_res.edits.len());
    assert_eq!(edit_form.new_text, get_res.article.text);
    assert!(!get_res.article.local);

    data.stop()
}

#[tokio::test]
async fn test_edit_local_article() -> MyResult<()> {
    let data = TestData::start().await;

    let beta_instance = data.alpha.follow_instance(&data.beta.hostname).await?;

    // create new article
    let create_form = CreateArticleData {
        title: "Manu_Chao".to_string(),
        text: TEST_ARTICLE_DEFAULT_TEXT.to_string(),
        summary: "create article".to_string(),
    };
    let create_res = data.beta.create_article(&create_form).await?;
    assert_eq!(create_form.title, create_res.article.title);
    assert!(create_res.article.local);

    // article should be federated to alpha
    let get_article_data = GetArticleData {
        title: Some(create_res.article.title.to_string()),
        instance_id: Some(beta_instance.id),
        id: None,
    };
    let get_res = data.alpha.get_article(get_article_data.clone()).await?;
    assert_eq!(create_res.article.title, get_res.article.title);
    assert_eq!(1, get_res.edits.len());
    assert!(!get_res.article.local);
    assert_eq!(create_res.article.text, get_res.article.text);

    // edit the article
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum 2\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: get_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = data.beta.edit_article(&edit_form).await?;
    assert_eq!(edit_res.article.text, edit_form.new_text);
    assert_eq!(edit_res.edits.len(), 2);
    assert!(edit_res.edits[0]
        .ap_id
        .to_string()
        .starts_with(&edit_res.article.ap_id.to_string()));

    // edit should be federated to alpha
    let get_res = data.alpha.get_article(get_article_data).await?;
    assert_eq!(edit_res.article.title, get_res.article.title);
    assert_eq!(edit_res.edits.len(), 2);
    assert_eq!(edit_res.article.text, get_res.article.text);

    data.stop()
}

#[tokio::test]
async fn test_edit_remote_article() -> MyResult<()> {
    let data = TestData::start().await;

    let beta_id_on_alpha = data.alpha.follow_instance(&data.beta.hostname).await?;
    let beta_id_on_gamma = data.gamma.follow_instance(&data.beta.hostname).await?;

    // create new article
    let create_form = CreateArticleData {
        title: "Manu_Chao".to_string(),
        text: TEST_ARTICLE_DEFAULT_TEXT.to_string(),
        summary: "create article".to_string(),
    };
    let create_res = data.beta.create_article(&create_form).await?;
    assert_eq!(&create_form.title, &create_res.article.title);
    assert!(create_res.article.local);

    // article should be federated to alpha and gamma
    let get_article_data_alpha = GetArticleData {
        title: Some(create_res.article.title.to_string()),
        instance_id: Some(beta_id_on_alpha.id),
        id: None,
    };
    let get_res = data
        .alpha
        .get_article(get_article_data_alpha.clone())
        .await?;
    assert_eq!(create_res.article.title, get_res.article.title);
    assert_eq!(1, get_res.edits.len());
    assert!(!get_res.article.local);

    let get_article_data_gamma = GetArticleData {
        title: Some(create_res.article.title.to_string()),
        instance_id: Some(beta_id_on_gamma.id),
        id: None,
    };
    let get_res = data
        .gamma
        .get_article(get_article_data_gamma.clone())
        .await?;
    assert_eq!(create_res.article.title, get_res.article.title);
    assert_eq!(create_res.article.text, get_res.article.text);

    let edit_form = EditArticleData {
        article_id: get_res.article.id,
        new_text: "Lorem Ipsum 2\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: get_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = data.alpha.edit_article(&edit_form).await?;
    assert_eq!(edit_form.new_text, edit_res.article.text);
    assert_eq!(2, edit_res.edits.len());
    assert!(!edit_res.article.local);
    assert!(edit_res.edits[0]
        .ap_id
        .to_string()
        .starts_with(&edit_res.article.ap_id.to_string()));

    // edit should be federated to beta and gamma
    let get_res = data.alpha.get_article(get_article_data_alpha).await?;
    assert_eq!(edit_res.article.title, get_res.article.title);
    assert_eq!(edit_res.edits.len(), 2);
    assert_eq!(edit_res.article.text, get_res.article.text);

    let get_res = data.gamma.get_article(get_article_data_gamma).await?;
    assert_eq!(edit_res.article.title, get_res.article.title);
    assert_eq!(edit_res.edits.len(), 2);
    assert_eq!(edit_res.article.text, get_res.article.text);

    data.stop()
}

#[tokio::test]
async fn test_local_edit_conflict() -> MyResult<()> {
    let data = TestData::start().await;

    // create new article
    let create_form = CreateArticleData {
        title: "Manu_Chao".to_string(),
        text: TEST_ARTICLE_DEFAULT_TEXT.to_string(),
        summary: "create article".to_string(),
    };
    let create_res = data.alpha.create_article(&create_form).await?;
    assert_eq!(create_form.title, create_res.article.title);
    assert!(create_res.article.local);

    // one user edits article
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: create_res.latest_version.clone(),
        resolve_conflict_id: None,
    };
    let edit_res = data.alpha.edit_article(&edit_form).await?;
    assert_eq!(edit_res.article.text, edit_form.new_text);
    assert_eq!(2, edit_res.edits.len());

    // another user edits article, without being aware of previous edit
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Ipsum Lorem\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: create_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = data
        .alpha
        .edit_article_with_conflict(&edit_form)
        .await?
        .unwrap();
    assert_eq!("<<<<<<< ours\nIpsum Lorem\n||||||| original\nsome\nexample\ntext\n=======\nLorem Ipsum\n>>>>>>> theirs\n", edit_res.three_way_merge);

    let conflicts = data.alpha.get_conflicts().await?;
    assert_eq!(1, conflicts.len());
    assert_eq!(conflicts[0], edit_res);

    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum and Ipsum Lorem\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: edit_res.previous_version_id,
        resolve_conflict_id: Some(edit_res.id),
    };
    let edit_res = data.alpha.edit_article(&edit_form).await?;
    assert_eq!(edit_form.new_text, edit_res.article.text);

    let conflicts = data.alpha.get_conflicts().await?;
    assert_eq!(0, conflicts.len());

    data.stop()
}

#[tokio::test]
async fn test_federated_edit_conflict() -> MyResult<()> {
    let data = TestData::start().await;

    let beta_id_on_alpha = data.alpha.follow_instance(&data.beta.hostname).await?;

    // create new article
    let create_form = CreateArticleData {
        title: "Manu_Chao".to_string(),
        text: TEST_ARTICLE_DEFAULT_TEXT.to_string(),
        summary: "create article".to_string(),
    };
    let create_res = data.beta.create_article(&create_form).await?;
    assert_eq!(create_form.title, create_res.article.title);
    assert!(create_res.article.local);

    // fetch article to gamma
    let resolve_res: ArticleView = data
        .gamma
        .resolve_article(create_res.article.ap_id.inner().clone())
        .await?;
    assert_eq!(create_res.article.text, resolve_res.article.text);

    // alpha edits article
    let get_article_data = GetArticleData {
        title: Some(create_form.title.to_string()),
        instance_id: Some(beta_id_on_alpha.id),
        id: None,
    };
    let get_res = data.alpha.get_article(get_article_data).await?;
    assert_eq!(&create_res.edits.len(), &get_res.edits.len());
    assert_eq!(&create_res.edits[0].hash, &get_res.edits[0].hash);
    let edit_form = EditArticleData {
        article_id: get_res.article.id,
        new_text: "Lorem Ipsum\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: create_res.latest_version.clone(),
        resolve_conflict_id: None,
    };
    let edit_res = data.alpha.edit_article(&edit_form).await?;
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
        article_id: resolve_res.article.id,
        new_text: "aaaa\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: create_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = data.gamma.edit_article(&edit_form).await?;
    assert_ne!(edit_form.new_text, edit_res.article.text);
    assert_eq!(1, edit_res.edits.len());
    assert!(!edit_res.article.local);

    let conflicts = data.gamma.get_conflicts().await?;
    assert_eq!(1, conflicts.len());

    // resolve the conflict
    let edit_form = EditArticleData {
        article_id: resolve_res.article.id,
        new_text: "aaaa\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: conflicts[0].previous_version_id.clone(),
        resolve_conflict_id: Some(conflicts[0].id.clone()),
    };
    let edit_res = data.gamma.edit_article(&edit_form).await?;
    assert_eq!(edit_form.new_text, edit_res.article.text);
    assert_eq!(3, edit_res.edits.len());

    let conflicts = data.gamma.get_conflicts().await?;
    assert_eq!(0, conflicts.len());

    data.stop()
}

#[tokio::test]
async fn test_overlapping_edits_no_conflict() -> MyResult<()> {
    let data = TestData::start().await;

    // create new article
    let create_form = CreateArticleData {
        title: "Manu_Chao".to_string(),
        text: TEST_ARTICLE_DEFAULT_TEXT.to_string(),
        summary: "create article".to_string(),
    };
    let create_res = data.alpha.create_article(&create_form).await?;
    assert_eq!(create_form.title, create_res.article.title);
    assert!(create_res.article.local);

    // one user edits article
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "my\nexample\ntext\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: create_res.latest_version.clone(),
        resolve_conflict_id: None,
    };
    let edit_res = data.alpha.edit_article(&edit_form).await?;
    assert_eq!(edit_res.article.text, edit_form.new_text);
    assert_eq!(2, edit_res.edits.len());

    // another user edits article, without being aware of previous edit
    let edit_form = EditArticleData {
        article_id: create_res.article.id,
        new_text: "some\nexample\narticle\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: create_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = data.alpha.edit_article(&edit_form).await?;
    let conflicts = data.alpha.get_conflicts().await?;
    assert_eq!(0, conflicts.len());
    assert_eq!(3, edit_res.edits.len());
    assert_eq!("my\nexample\narticle\n", edit_res.article.text);

    data.stop()
}

#[tokio::test]
async fn test_fork_article() -> MyResult<()> {
    let data = TestData::start().await;

    // create article
    let create_form = CreateArticleData {
        title: "Manu_Chao".to_string(),
        text: TEST_ARTICLE_DEFAULT_TEXT.to_string(),
        summary: "create article".to_string(),
    };
    let create_res = data.alpha.create_article(&create_form).await?;
    assert_eq!(create_form.title, create_res.article.title);
    assert!(create_res.article.local);

    // fetch on beta
    let resolve_res = data
        .beta
        .resolve_article(create_res.article.ap_id.into_inner())
        .await?;
    let resolved_article = resolve_res.article;
    assert_eq!(create_res.edits.len(), resolve_res.edits.len());

    // fork the article to local instance
    let fork_form = ForkArticleData {
        article_id: resolved_article.id,
    };
    let fork_res = data.beta.fork_article(&fork_form).await?;
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

    let beta_instance = data.beta.get_local_instance().await?;
    assert_eq!(forked_article.instance_id, beta_instance.instance.id);

    // now search returns two articles for this title (original and forked)
    let search_form = SearchArticleData {
        query: create_form.title.clone(),
    };
    let search_res = data.beta.search(&search_form).await?;
    assert_eq!(2, search_res.len());

    data.stop()
}

#[tokio::test]
async fn test_user_registration_login() -> MyResult<()> {
    let data = TestData::start().await;
    let username = "my_user";
    let password = "hunter2";
    let register_data = RegisterUserData {
        username: username.to_string(),
        password: password.to_string(),
    };
    data.alpha.register(register_data).await?;

    let login_data = LoginUserData {
        username: username.to_string(),
        password: "asd123".to_string(),
    };
    let invalid_login = data.alpha.login(login_data).await;
    assert!(invalid_login.is_err());

    let login_data = LoginUserData {
        username: username.to_string(),
        password: password.to_string(),
    };
    data.alpha.login(login_data).await?;

    let my_profile = data.alpha.my_profile().await?;
    assert_eq!(username, my_profile.person.username);

    data.alpha.logout().await?;

    let my_profile_after_logout = data.alpha.my_profile().await;
    assert!(my_profile_after_logout.is_err());

    data.stop()
}
