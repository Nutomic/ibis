#![expect(clippy::unwrap_used)]

mod common;

use crate::common::{TEST_ARTICLE_DEFAULT_TEXT, TestData};
use anyhow::Result;
use ibis_api_client::{
    article::{
        CreateArticleParams,
        EditArticleParams,
        ForkArticleParams,
        GetArticleParams,
        ListArticlesParams,
        ProtectArticleParams,
    },
    comment::{CreateCommentParams, EditCommentParams},
    instance::SearchArticleParams,
    user::{GetUserParams, LoginUserParams, RegisterUserParams},
};
use ibis_database::common::{
    article::ArticleView,
    notifications::ApiNotificationData,
    utils::extract_domain,
};
use pretty_assertions::assert_eq;
use retry_future::{LinearRetryStrategy, RetryFuture, RetryPolicy};
use std::time::Duration;
use test_context::test_context;
use tokio::time::sleep;
use url::Url;

fn create_test_article_params() -> CreateArticleParams {
    CreateArticleParams {
        title: "Manu Chao".to_string(),
        text: TEST_ARTICLE_DEFAULT_TEXT.to_string(),
        summary: "create article".to_string(),
        instance_id: None,
    }
}

#[test_context(TestData)]
#[tokio::test]
async fn api_test_create_read_and_edit_local_article(
    TestData(alpha, beta, _): &mut TestData,
) -> Result<()> {
    // create article
    let create_params = create_test_article_params();
    let create_res = alpha.create_article(&create_params).await.unwrap();
    assert_eq!(create_params.title, create_res.article.title);
    assert!(create_res.article.local);

    // now article can be read
    let get_article_data = GetArticleParams {
        title: Some(create_res.article.title.clone()),
        domain: None,
        id: None,
    };
    let get_res = alpha.get_article(get_article_data.clone()).await.unwrap();
    assert_eq!(create_params.title, get_res.article.title);
    assert_eq!(TEST_ARTICLE_DEFAULT_TEXT, get_res.article.text);
    assert!(get_res.article.local);

    // error on article which wasnt federated
    let not_found = beta.get_article(get_article_data.clone()).await;
    assert!(not_found.is_err());

    // edit article
    let edit_params = EditArticleParams {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum 2\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: get_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = alpha
        .edit_article_without_conflict(&edit_params)
        .await
        .unwrap();
    assert_eq!(edit_params.new_text, edit_res.article.text);
    let edits = alpha.get_article_edits(edit_res.article.id).await.unwrap();
    assert_eq!(2, edits.len());
    assert_eq!(edit_params.summary, edits[1].edit.summary);

    let search_params = SearchArticleParams {
        query: create_params.title.clone(),
    };
    let search_res = alpha.search(&search_params).await.unwrap();
    assert_eq!(1, search_res.len());
    assert_eq!(edit_res.article, search_res[0]);

    let list_articles = alpha
        .list_articles(ListArticlesParams {
            only_local: Some(false),
            instance_id: None,
            include_removed: None,
        })
        .await
        .unwrap();
    assert_eq!(2, list_articles.len());
    assert_eq!(edit_res.article, list_articles[0]);

    Ok(())
}

#[test_context(TestData)]
#[tokio::test]
async fn api_test_create_duplicate_article(TestData(alpha, _, _): &mut TestData) -> Result<()> {
    // create article
    let create_params = create_test_article_params();
    let create_res = alpha.create_article(&create_params).await.unwrap();
    assert_eq!(create_params.title, create_res.article.title);
    assert!(create_res.article.local);

    let create_res = alpha.create_article(&create_params).await;
    assert!(create_res.is_err());

    Ok(())
}

#[test_context(TestData)]
#[tokio::test]
async fn api_test_follow_instance(TestData(alpha, beta, _): &mut TestData) -> Result<()> {
    // check initial state
    let alpha_follows = alpha.get_follows().await.unwrap();
    assert_eq!(0, alpha_follows.len());
    let beta_follows = beta.get_follows().await.unwrap();
    assert_eq!(0, beta_follows.len());

    alpha
        .follow_instance_with_resolve(&beta.hostname)
        .await
        .unwrap();

    // check that follow was federated
    let alpha_follows = alpha.get_follows().await.unwrap();
    assert_eq!(1, alpha_follows.len());
    assert!(!alpha_follows[0].pending);
    let beta_site = beta.site().await.unwrap();
    assert_eq!(beta_site.instance.ap_id, alpha_follows[0].instance.ap_id);

    // unfollow
    alpha
        .follow_instance(alpha_follows[0].instance.id, false)
        .await
        .unwrap();
    let alpha_follows = alpha.get_follows().await.unwrap();
    assert_eq!(0, alpha_follows.len());

    Ok(())
}

#[test_context(TestData)]
#[tokio::test]
async fn api_test_synchronize_articles(TestData(alpha, beta, _): &mut TestData) -> Result<()> {
    // create article on alpha
    let create_params = create_test_article_params();
    let create_res = alpha.create_article(&create_params).await.unwrap();
    assert_eq!(create_params.title, create_res.article.title);
    let edits = alpha
        .get_article_edits(create_res.article.id)
        .await
        .unwrap();
    assert_eq!(1, edits.len());
    assert!(create_res.article.local);

    // edit the article
    let edit_params = EditArticleParams {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum 2\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: create_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = alpha
        .edit_article_without_conflict(&edit_params)
        .await
        .unwrap();

    // fetch alpha instance on beta, articles are also fetched automatically
    let instance = beta
        .resolve_instance(Url::parse(&format!("http://{}", &alpha.hostname))?)
        .await
        .unwrap();

    let get_article_data = GetArticleParams {
        title: Some(create_res.article.title.clone()),
        ..Default::default()
    };

    // try to read remote article by name, fails without domain
    let get_res = beta.get_article(get_article_data.clone()).await;
    assert!(get_res.is_err());

    // get the article with instance id and compare
    let get_res = RetryFuture::new(
        || async {
            let get_article_data = GetArticleParams {
                title: Some(create_res.article.title.clone()),
                domain: Some(instance.domain.clone()),
                id: None,
            };
            match beta.get_article(get_article_data).await {
                Err(_) => Err(RetryPolicy::<String>::Retry(None)),
                Ok(a) if a.latest_version != edit_res.latest_version => {
                    Err(RetryPolicy::Retry(None))
                }
                Ok(a) => Ok(a),
            }
        },
        LinearRetryStrategy::new(),
    )
    .await?;
    let beta_edits = beta.get_article_edits(get_res.article.id).await.unwrap();
    assert_eq!(create_res.article.ap_id, get_res.article.ap_id);
    assert_eq!(create_params.title, get_res.article.title);
    assert_eq!(2, beta_edits.len());
    assert_eq!(edit_params.new_text, get_res.article.text);
    assert!(!get_res.article.local);

    Ok(())
}

#[test_context(TestData)]
#[tokio::test]
async fn api_test_edit_local_article(TestData(alpha, beta, _): &mut TestData) -> Result<()> {
    let beta_instance = alpha
        .follow_instance_with_resolve(&beta.hostname)
        .await
        .unwrap();

    // create new article
    let create_params = create_test_article_params();
    let create_res = beta.create_article(&create_params).await.unwrap();
    assert_eq!(create_params.title, create_res.article.title);
    assert!(create_res.article.local);

    // article should be federated to alpha
    let get_article_data = GetArticleParams {
        title: Some(create_res.article.title.to_string()),
        domain: Some(beta_instance.domain),
        id: None,
    };
    let get_res = alpha.get_article(get_article_data.clone()).await.unwrap();
    let edits = alpha.get_article_edits(get_res.article.id).await.unwrap();
    assert_eq!(create_res.article.title, get_res.article.title);
    assert_eq!(1, edits.len());
    assert!(!get_res.article.local);
    assert_eq!(create_res.article.text, get_res.article.text);

    // edit the article
    let edit_params = EditArticleParams {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum 2\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: get_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = beta
        .edit_article_without_conflict(&edit_params)
        .await
        .unwrap();
    let edits = beta.get_article_edits(edit_res.article.id).await.unwrap();
    assert_eq!(edit_res.article.text, edit_params.new_text);
    assert_eq!(edits.len(), 2);
    assert!(
        edits[0]
            .edit
            .ap_id
            .to_string()
            .starts_with(&edit_res.article.ap_id.to_string())
    );

    // edit should be federated to alpha
    let get_res = alpha.get_article(get_article_data).await.unwrap();
    let edits = alpha.get_article_edits(get_res.article.id).await.unwrap();
    assert_eq!(edit_res.article.title, get_res.article.title);
    assert_eq!(edits.len(), 2);
    assert_eq!(edit_res.article.text, get_res.article.text);

    Ok(())
}

#[test_context(TestData)]
#[tokio::test]
async fn api_test_edit_remote_article(TestData(alpha, beta, gamma): &mut TestData) -> Result<()> {
    let beta_id_on_alpha = alpha
        .follow_instance_with_resolve(&beta.hostname)
        .await
        .unwrap();
    let beta_id_on_gamma = gamma
        .follow_instance_with_resolve(&beta.hostname)
        .await
        .unwrap();

    // create new article
    let create_params = create_test_article_params();
    let create_res = beta.create_article(&create_params).await.unwrap();
    assert_eq!(&create_params.title, &create_res.article.title);
    assert!(create_res.article.local);

    // article should be federated to alpha and gamma
    let get_article_data_alpha = GetArticleParams {
        title: Some(create_res.article.title.to_string()),
        domain: Some(beta_id_on_alpha.domain),
        id: None,
    };
    let get_res = alpha
        .get_article(get_article_data_alpha.clone())
        .await
        .unwrap();
    assert_eq!(create_res.article.title, get_res.article.title);
    let edits = alpha.get_article_edits(get_res.article.id).await.unwrap();
    assert_eq!(1, edits.len());
    assert!(!get_res.article.local);

    let get_article_data_gamma = GetArticleParams {
        title: Some(create_res.article.title.to_string()),
        domain: Some(beta_id_on_gamma.domain),
        id: None,
    };
    let get_res = gamma
        .get_article(get_article_data_gamma.clone())
        .await
        .unwrap();
    assert_eq!(create_res.article.title, get_res.article.title);
    assert_eq!(create_res.article.text, get_res.article.text);

    let edit_params = EditArticleParams {
        article_id: get_res.article.id,
        new_text: "Lorem Ipsum 2\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: get_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = alpha
        .edit_article_without_conflict(&edit_params)
        .await
        .unwrap();
    assert_eq!(edit_params.new_text, edit_res.article.text);
    let edits = alpha.get_article_edits(edit_res.article.id).await.unwrap();
    assert_eq!(2, edits.len());
    assert!(!edit_res.article.local);
    assert!(
        edits[0]
            .edit
            .ap_id
            .to_string()
            .starts_with(&edit_res.article.ap_id.to_string())
    );

    // edit should be federated to beta and gamma
    let get_res = beta.get_article(get_article_data_alpha).await.unwrap();
    let edits = beta.get_article_edits(get_res.article.id).await.unwrap();
    assert_eq!(edit_res.article.title, get_res.article.title);
    assert_eq!(edits.len(), 2);
    assert_eq!(edit_res.article.text, get_res.article.text);

    let get_res = gamma.get_article(get_article_data_gamma).await.unwrap();
    let edits = gamma.get_article_edits(edit_res.article.id).await.unwrap();
    assert_eq!(edit_res.article.title, get_res.article.title);
    assert_eq!(edits.len(), 2);
    assert_eq!(edit_res.article.text, get_res.article.text);

    Ok(())
}

#[test_context(TestData)]
#[tokio::test]
async fn api_test_local_edit_conflict(TestData(alpha, _, _): &mut TestData) -> Result<()> {
    // create new article
    let create_params = create_test_article_params();
    let create_res = alpha.create_article(&create_params).await.unwrap();
    assert_eq!(create_params.title, create_res.article.title);
    assert!(create_res.article.local);

    // one user edits article
    let edit_params = EditArticleParams {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: create_res.latest_version.clone(),
        resolve_conflict_id: None,
    };
    let edit_res = alpha
        .edit_article_without_conflict(&edit_params)
        .await
        .unwrap();
    let edits = alpha.get_article_edits(edit_res.article.id).await.unwrap();
    assert_eq!(edit_res.article.text, edit_params.new_text);
    assert_eq!(2, edits.len());

    // another user edits article, without being aware of previous edit
    let edit_params = EditArticleParams {
        article_id: create_res.article.id,
        new_text: "Ipsum Lorem\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: create_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = alpha.edit_article(&edit_params).await.unwrap().unwrap();
    assert_eq!(
        "<<<<<<< ours\nIpsum Lorem\n||||||| original\nsome example text\n=======\nLorem Ipsum\n>>>>>>> theirs\n",
        edit_res.three_way_merge
    );

    let notifications = alpha.notifications_list().await.unwrap();
    assert_eq!(1, notifications.len());
    let ApiNotificationData::EditConflict {
        conflict_id,
        summary,
    } = &notifications[0].data
    else {
        panic!()
    };
    assert_eq!(conflict_id, &edit_res.id);
    assert_eq!(summary, &edit_res.summary);

    let edit_params = EditArticleParams {
        article_id: create_res.article.id,
        new_text: "Lorem Ipsum and Ipsum Lorem\n".to_string(),
        summary: "summary".to_string(),
        previous_version_id: edit_res.previous_version_id,
        resolve_conflict_id: Some(edit_res.id),
    };
    let edit_res = alpha
        .edit_article_without_conflict(&edit_params)
        .await
        .unwrap();
    assert_eq!(edit_params.new_text, edit_res.article.text);

    assert_eq!(0, alpha.notifications_count().await.unwrap());

    Ok(())
}

#[test_context(TestData)]
#[tokio::test]
#[ignore]
async fn api_test_federated_edit_conflict(
    TestData(alpha, beta, gamma): &mut TestData,
) -> Result<()> {
    let beta_id_on_alpha = alpha
        .follow_instance_with_resolve(&beta.hostname)
        .await
        .unwrap();

    // create new article
    let create_params = create_test_article_params();
    let create_res = beta.create_article(&create_params).await.unwrap();
    let beta_edits = beta.get_article_edits(create_res.article.id).await.unwrap();
    assert_eq!(create_params.title, create_res.article.title);
    assert!(create_res.article.local);

    // fetch article to gamma
    let resolve_res: ArticleView = gamma
        .resolve_article(create_res.article.ap_id.inner().clone())
        .await
        .unwrap();
    assert_eq!(create_res.article.text, resolve_res.article.text);

    // alpha edits article
    let get_article_data = GetArticleParams {
        title: Some(create_params.title.to_string()),
        domain: Some(beta_id_on_alpha.domain),
        id: None,
    };
    let get_res = alpha.get_article(get_article_data).await.unwrap();
    let alpha_edits = alpha.get_article_edits(get_res.article.id).await.unwrap();
    assert_eq!(&beta_edits.len(), &alpha_edits.len());
    assert_eq!(&beta_edits[0].edit.hash, &alpha_edits[0].edit.hash);
    let edit_params = EditArticleParams {
        article_id: get_res.article.id,
        new_text: "first edit\n".to_string(),
        summary: "first edit".to_string(),
        previous_version_id: create_res.latest_version.clone(),
        resolve_conflict_id: None,
    };
    let edit_res = alpha
        .edit_article_without_conflict(&edit_params)
        .await
        .unwrap();
    let alpha_edits = alpha.get_article_edits(get_res.article.id).await.unwrap();
    assert_eq!(edit_res.article.text, edit_params.new_text);
    assert_eq!(2, alpha_edits.len());
    assert!(!edit_res.article.local);
    assert!(
        alpha_edits[1]
            .edit
            .ap_id
            .to_string()
            .starts_with(&edit_res.article.ap_id.to_string())
    );

    // gamma also edits, as its not the latest version there is a conflict. local version should
    // not be updated with this conflicting version, instead user needs to handle the conflict
    let edit_params = EditArticleParams {
        article_id: resolve_res.article.id,
        new_text: "second edit\n".to_string(),
        summary: "second edit".to_string(),
        previous_version_id: create_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = gamma
        .edit_article_without_conflict(&edit_params)
        .await
        .unwrap();
    let gamma_edits = gamma.get_article_edits(edit_res.article.id).await.unwrap();
    assert_ne!(edit_params.new_text, edit_res.article.text);
    assert_eq!(3, gamma_edits.len());
    assert!(gamma_edits[2].edit.pending);
    assert!(!edit_res.article.local);

    assert_eq!(1, gamma.notifications_count().await.unwrap());
    let notifications = gamma.notifications_list().await.unwrap();
    assert_eq!(1, notifications.len());
    let ApiNotificationData::EditConflict {
        conflict_id,
        summary: _,
    } = &notifications[0].data
    else {
        panic!()
    };

    let conflict = gamma.get_conflict(*conflict_id).await?;

    // resolve the conflict
    let edit_params = EditArticleParams {
        article_id: resolve_res.article.id,
        new_text: "second edit\n".to_string(),
        summary: "resolve conflict".to_string(),
        previous_version_id: conflict.previous_version_id.clone(),
        resolve_conflict_id: Some(conflict.id),
    };
    let edit_res = gamma
        .edit_article_without_conflict(&edit_params)
        .await
        .unwrap();
    let gamma_edits = gamma.get_article_edits(edit_res.article.id).await.unwrap();
    assert_eq!(edit_params.new_text, edit_res.article.text);
    assert_eq!(3, gamma_edits.len());
    assert!(gamma_edits.iter().all(|e| !e.edit.pending));

    assert_eq!(0, gamma.notifications_count().await.unwrap());
    let notifications = gamma.notifications_list().await.unwrap();
    assert_eq!(0, notifications.len());

    Ok(())
}

#[test_context(TestData)]
#[tokio::test]
async fn api_test_overlapping_edits_no_conflict(
    TestData(alpha, _, _): &mut TestData,
) -> Result<()> {
    // Create new article
    // Need to use multiple lines to provide enough context for diff/merge.
    // Also need to use long lines so that markdown paramsatting doesnt change line breaks.
    let create_params = CreateArticleParams {
        title: "Manu Chao".to_string(),
        text: r#"1 Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod.
2 Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod.
3 Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod.
4 Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod.
"#
        .to_string(),
        summary: "create article".to_string(),
        instance_id: None,
    };
    let create_res = alpha.create_article(&create_params).await.unwrap();
    assert_eq!(create_params.title, create_res.article.title);
    assert!(create_res.article.local);

    // one user edits article
    let edit_params = EditArticleParams {
        article_id: create_res.article.id,
        new_text: r#"1 Lorem **changed** dolor sit amet consectetur adipiscing elit sed do eiusmod.
2 Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod.
3 Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod.
4 Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod.
"#
        .to_string(),
        summary: "summary".to_string(),
        previous_version_id: create_res.latest_version.clone(),
        resolve_conflict_id: None,
    };
    let edit_res = alpha
        .edit_article_without_conflict(&edit_params)
        .await
        .unwrap();
    let alpha_edits = alpha.get_article_edits(edit_res.article.id).await.unwrap();
    assert_eq!(edit_res.article.text, edit_params.new_text);
    assert_eq!(2, alpha_edits.len());

    // another user edits article, without being aware of previous edit
    let edit_params = EditArticleParams {
        article_id: create_res.article.id,
        new_text: r#"1 Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod.
2 Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod.
3 Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod.
4 Lorem **changed** dolor sit amet consectetur adipiscing elit sed do eiusmod.
"#
        .to_string(),
        summary: "summary".to_string(),
        previous_version_id: create_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = alpha
        .edit_article_without_conflict(&edit_params)
        .await
        .unwrap();
    let alpha_edits = alpha.get_article_edits(edit_res.article.id).await.unwrap();
    assert_eq!(0, alpha.notifications_count().await.unwrap());
    assert_eq!(3, alpha_edits.len());
    assert_eq!(
        r#"1 Lorem **changed** dolor sit amet consectetur adipiscing elit sed do eiusmod.
2 Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod.
3 Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod.
4 Lorem **changed** dolor sit amet consectetur adipiscing elit sed do eiusmod.
"#,
        edit_res.article.text
    );

    Ok(())
}

#[test_context(TestData)]
#[tokio::test]
async fn api_test_fork_article(TestData(alpha, beta, _): &mut TestData) -> Result<()> {
    // create article
    let create_params = create_test_article_params();
    let create_res = alpha.create_article(&create_params).await.unwrap();
    let create_edits = alpha
        .get_article_edits(create_res.article.id)
        .await
        .unwrap();
    assert_eq!(create_params.title, create_res.article.title);
    assert!(create_res.article.local);

    // fetch on beta
    let resolve_res = beta
        .resolve_article(create_res.article.ap_id.into())
        .await
        .unwrap();
    let resolved_article = resolve_res.article;
    let resolve_edits = beta.get_article_edits(resolved_article.id).await.unwrap();
    assert_eq!(create_edits.len(), resolve_edits.len());

    // fork the article to local instance
    let fork_params = ForkArticleParams {
        article_id: resolved_article.id,
        new_title: resolved_article.title.clone(),
    };
    let fork_res = beta.fork_article(&fork_params).await.unwrap();
    let forked_article = fork_res.article;
    let fork_edits = beta.get_article_edits(forked_article.id).await.unwrap();
    assert_eq!(resolved_article.title, forked_article.title);
    assert_eq!(resolved_article.text, forked_article.text);
    assert_eq!(resolve_edits.len(), fork_edits.len());
    assert_eq!(resolve_edits[0].edit.diff, fork_edits[0].edit.diff);
    assert_eq!(resolve_edits[0].edit.hash, fork_edits[0].edit.hash);
    assert_ne!(resolve_edits[0].edit.id, fork_edits[0].edit.id);
    assert_eq!(resolve_res.latest_version, fork_res.latest_version);
    assert_ne!(resolved_article.ap_id, forked_article.ap_id);
    assert!(forked_article.local);

    let beta_site = beta.site().await.unwrap();
    assert_eq!(forked_article.instance_id, beta_site.instance.id);

    // now search returns two articles for this title (original and forked)
    let search_params = SearchArticleParams {
        query: create_params.title.clone(),
    };
    let search_res = beta.search(&search_params).await.unwrap();
    assert_eq!(2, search_res.len());

    Ok(())
}

#[test_context(TestData)]
#[tokio::test]
async fn api_test_user_registration_login(TestData(alpha, _, _): &mut TestData) -> Result<()> {
    let username = "my_user";
    let password = "hunter22";
    let register_data = RegisterUserParams {
        username: username.to_string(),
        password: password.to_string(),
        email: None,
        confirm_password: password.to_string(),
    };
    alpha.register(register_data).await.unwrap();

    let login_data = LoginUserParams {
        username_or_email: username.to_string(),
        password: "asd123".to_string(),
    };
    let invalid_login = alpha.login(login_data).await;
    assert!(invalid_login.is_err());

    let login_data = LoginUserParams {
        username_or_email: username.to_string(),
        password: password.to_string(),
    };
    alpha.login(login_data).await.unwrap();

    let my_profile = alpha.site().await.unwrap().my_profile.unwrap();
    assert_eq!(username, my_profile.person.username);

    alpha.logout().await.unwrap();

    let my_profile_after_logout = alpha.site().await.unwrap().my_profile;
    assert!(my_profile_after_logout.is_none());

    Ok(())
}

#[test_context(TestData)]
#[tokio::test]
async fn api_test_user_profile(TestData(alpha, beta, _): &mut TestData) -> Result<()> {
    // Create an article and federate it, in order to federate the user who created it
    let create_params = create_test_article_params();
    let create_res = alpha.create_article(&create_params).await.unwrap();
    beta.resolve_article(create_res.article.ap_id.into())
        .await
        .unwrap();
    let domain = extract_domain(
        &alpha
            .site()
            .await
            .unwrap()
            .my_profile
            .unwrap()
            .person
            .ap_id
            .into(),
    );

    // Now we can fetch the remote user from local api
    let params = GetUserParams {
        name: "alpha".to_string(),
        domain: Some(domain),
    };
    let user = beta.get_user(params).await.unwrap();
    assert_eq!("alpha", user.username);
    assert!(!user.local);

    Ok(())
}

#[test_context(TestData)]
#[tokio::test]
async fn api_test_lock_article(TestData(alpha, _, gamma): &mut TestData) -> Result<()> {
    // create article
    let create_params = create_test_article_params();
    let create_res = alpha.create_article(&create_params).await.unwrap();
    assert!(!create_res.article.protected);

    // lock from normal user fails
    let lock_params = ProtectArticleParams {
        article_id: create_res.article.id,
        protected: true,
    };
    let lock_res = alpha.protect_article(&lock_params).await;
    assert!(lock_res.is_err());

    // login as admin to lock article
    let params = LoginUserParams {
        username_or_email: "ibis".to_string(),
        password: "ibis".to_string(),
    };
    alpha.login(params).await.unwrap();
    let lock_res = alpha.protect_article(&lock_params).await.unwrap();
    assert!(lock_res.protected);

    let resolve_res: ArticleView = gamma
        .resolve_article(create_res.article.ap_id.inner().clone())
        .await
        .unwrap();
    let edit_params = EditArticleParams {
        article_id: resolve_res.article.id,
        new_text: "test".to_string(),
        summary: "test".to_string(),
        previous_version_id: resolve_res.latest_version,
        resolve_conflict_id: None,
    };
    let edit_res = gamma.edit_article_without_conflict(&edit_params).await;
    assert!(edit_res.is_none());

    Ok(())
}

#[test_context(TestData)]
#[tokio::test]
async fn api_test_synchronize_instances(TestData(alpha, beta, gamma): &mut TestData) -> Result<()> {
    // fetch alpha instance on beta
    beta.resolve_instance(Url::parse(&format!("http://{}", &alpha.hostname))?)
        .await
        .unwrap();
    let beta_instances = beta.list_instances().await.unwrap();
    assert_eq!(2, beta_instances.len());

    // fetch beta instance on gamma
    gamma
        .resolve_instance(Url::parse(&format!("http://{}", &beta.hostname))?)
        .await
        .unwrap();

    // wait until instance collection is fetched
    let gamma_instances = RetryFuture::new(
        || async {
            match gamma.list_instances().await {
                Err(_) => Err(RetryPolicy::<String>::Retry(None)),
                Ok(i) if i.len() < 3 => Err(RetryPolicy::Retry(None)),
                Ok(i) => Ok(i),
            }
        },
        LinearRetryStrategy::new(),
    )
    .await?;

    // now gamma also knows about alpha
    assert_eq!(3, gamma_instances.len());
    assert!(
        gamma_instances
            .iter()
            .any(|i| i.instance.domain == alpha.hostname)
    );

    Ok(())
}

#[test_context(TestData)]
#[tokio::test]
async fn api_test_remove_article(TestData(alpha, beta, _): &mut TestData) -> Result<()> {
    beta.follow_instance_with_resolve(&alpha.hostname)
        .await
        .unwrap();

    // create article
    let create_params = create_test_article_params();
    let create_res = alpha.create_article(&create_params).await.unwrap();

    let list_alpha = alpha.list_articles(Default::default()).await.unwrap();
    let article_to_remove_id = list_alpha[0].id;
    // count also includes auto-created main page
    assert_eq!(2, list_alpha.len());
    assert_eq!(article_to_remove_id, create_res.article.id);
    let list_beta = beta.list_articles(Default::default()).await.unwrap();
    // count also includes main pages from alpha and beta
    assert_eq!(3, list_beta.len());
    assert_eq!(create_res.article.ap_id, list_beta[0].ap_id);

    // login as admin to remove article
    let params = LoginUserParams {
        username_or_email: "ibis".to_string(),
        password: "ibis".to_string(),
    };
    alpha.login(params).await.unwrap();

    alpha
        .remove_article(article_to_remove_id, true)
        .await
        .unwrap();

    let params = GetArticleParams {
        title: Some(create_res.article.title),
        domain: Some(create_res.instance.domain),
        ..Default::default()
    };

    // cannot get the article
    sleep(Duration::from_secs(1)).await;
    assert!(beta.get_article(params.clone()).await.is_err());
    let list_beta = beta.list_articles(Default::default()).await?;
    assert_eq!(2, list_beta.len());

    // except as admin with include_removed
    let list_all = alpha
        .list_articles(ListArticlesParams {
            include_removed: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(2, list_all.len());

    // restore article
    alpha
        .remove_article(article_to_remove_id, false)
        .await
        .unwrap();

    // now it can be viewed again
    assert!(alpha.get_article(params).await.is_ok());
    let list_beta = beta.list_articles(Default::default()).await?;
    assert_eq!(3, list_beta.len());

    Ok(())
}

#[test_context(TestData)]
#[tokio::test]
async fn api_test_comment_create_edit(TestData(alpha, beta, gamma): &mut TestData) -> Result<()> {
    beta.follow_instance_with_resolve(&alpha.hostname)
        .await
        .unwrap();
    gamma
        .follow_instance_with_resolve(&alpha.hostname)
        .await
        .unwrap();

    // create article
    let params = create_test_article_params();
    let alpha_article = alpha.create_article(&params).await.unwrap();

    // fetch article on beta and create comment
    let beta_article = beta
        .resolve_article(alpha_article.article.ap_id.inner().clone())
        .await
        .unwrap();
    assert!(!beta_article.article.local);
    let params = CreateCommentParams {
        content: "top comment".to_string(),
        article_id: beta_article.article.id,
        parent_id: None,
    };
    let top_comment = beta.create_comment(&params).await.unwrap().comment;
    assert_eq!(top_comment.content, params.content);
    assert_eq!(top_comment.article_id, beta_article.article.id);
    assert_eq!(top_comment.depth, 0);
    assert!(top_comment.parent_id.is_none());
    assert!(top_comment.local);
    assert!(!top_comment.deleted);
    assert!(top_comment.updated.is_none());
    sleep(Duration::from_secs(1)).await;

    // now create child comment on alpha
    let get_params = GetArticleParams {
        title: Some(alpha_article.article.title),
        domain: Some(alpha.hostname.clone()),
        ..Default::default()
    };
    let article = beta.get_article(get_params.clone()).await.unwrap();
    assert_eq!(1, article.comments.len());

    // now create child comment on alpha
    sleep(Duration::from_secs(1)).await;
    let article = alpha.get_article(get_params.clone()).await.unwrap();
    assert_eq!(1, article.comments.len());
    let params = CreateCommentParams {
        content: "child comment".to_string(),
        article_id: article.article.id,
        parent_id: Some(article.comments[0].comment.id),
    };
    let child_comment = alpha.create_comment(&params).await.unwrap().comment;
    assert_eq!(child_comment.parent_id, Some(top_comment.id));
    assert_eq!(child_comment.depth, 1);

    // edit comment text
    let edit_params = EditCommentParams {
        id: child_comment.id,
        content: Some("edited comment".to_string()),
        deleted: None,
    };
    let edited_comment = alpha.edit_comment(&edit_params).await.unwrap().comment;
    assert_eq!(edited_comment.article_id, article.article.id);
    assert_eq!(Some(&edited_comment.content), edit_params.content.as_ref());

    let beta_comments = beta.get_article(get_params.clone()).await.unwrap().comments;
    assert_eq!(2, beta_comments.len());
    assert_eq!(beta_comments[1].comment.content, top_comment.content);
    assert_eq!(
        Some(&beta_comments[0].comment.content),
        edit_params.content.as_ref()
    );

    let gamma_comments = gamma.get_article(get_params).await.unwrap().comments;
    assert_eq!(2, gamma_comments.len());
    assert_eq!(edited_comment.content, gamma_comments[0].comment.content);

    Ok(())
}

#[test_context(TestData)]
#[tokio::test]
async fn api_test_comment_delete_restore(TestData(alpha, beta, _): &mut TestData) -> Result<()> {
    beta.follow_instance_with_resolve(&alpha.hostname)
        .await
        .unwrap();

    // create article and comment
    let params = create_test_article_params();
    let alpha_article = alpha.create_article(&params).await.unwrap();

    let params = CreateCommentParams {
        content: "my comment".to_string(),
        article_id: alpha_article.article.id,
        parent_id: None,
    };
    let comment = alpha.create_comment(&params).await.unwrap();
    let get_params = GetArticleParams {
        title: Some(alpha_article.article.title),
        domain: Some(alpha.hostname.clone()),
        ..Default::default()
    };

    // delete comment
    let mut params = EditCommentParams {
        id: comment.comment.id,
        deleted: Some(true),
        content: None,
    };
    alpha.edit_comment(&params).await.unwrap();
    let alpha_comments = alpha
        .get_article(get_params.clone())
        .await
        .unwrap()
        .comments;
    assert!(alpha_comments[0].comment.deleted);
    assert!(alpha_comments[0].comment.content.is_empty());
    sleep(Duration::from_secs(1)).await;

    // check that comment is deleted on beta
    let beta_comments = beta.get_article(get_params.clone()).await.unwrap().comments;
    assert_eq!(comment.comment.ap_id, beta_comments[0].comment.ap_id);
    assert!(beta_comments[0].comment.deleted);
    assert!(beta_comments[0].comment.content.is_empty());

    // restore comment
    params.deleted = Some(false);
    alpha.edit_comment(&params).await.unwrap();
    let alpha_comments = alpha
        .get_article(get_params.clone())
        .await
        .unwrap()
        .comments;
    assert!(!alpha_comments[0].comment.deleted);
    assert!(!alpha_comments[0].comment.content.is_empty());
    sleep(Duration::from_secs(1)).await;

    // check that comment is restored on beta
    let beta_comments = beta.get_article(get_params).await.unwrap().comments;
    assert!(!beta_comments[0].comment.deleted);
    assert!(!beta_comments[0].comment.content.is_empty());

    Ok(())
}

#[test_context(TestData)]
#[tokio::test]
async fn api_test_create_remote_article(TestData(alpha, beta, _): &mut TestData) -> Result<()> {
    let beta_instance = alpha.follow_instance_with_resolve(&beta.hostname).await?;

    // create article
    let create_params = CreateArticleParams {
        title: "Remote Test".to_string(),
        text: TEST_ARTICLE_DEFAULT_TEXT.to_string(),
        summary: "create article".to_string(),
        instance_id: Some(beta_instance.id),
    };
    let create_res = alpha.create_article(&create_params).await.unwrap();
    assert_eq!(create_params.title, create_res.article.title);
    assert!(!create_res.article.local);
    // Federation is synchronous so by the time api call returns it is already federated back and
    // not pending anymore.
    assert!(!create_res.article.pending);
    assert!(!create_res.article.local);
    assert_eq!(beta_instance.ap_id, create_res.instance.ap_id);

    // Check article shown on beta
    let get_article_data = GetArticleParams {
        title: Some(create_res.article.title.clone()),
        ..Default::default()
    };
    let beta_article = beta.get_article(get_article_data.clone()).await?;
    let site_view = beta.site().await?;
    assert!(!beta_article.article.pending);
    assert!(beta_article.article.local);
    assert_eq!(site_view.instance, beta_article.instance);
    assert_eq!(create_res.article.title, beta_article.article.title);
    assert_eq!(create_res.article.ap_id, beta_article.article.ap_id);

    // Article not pending anymore on alpha
    let get_article_data = GetArticleParams {
        id: Some(create_res.article.id),
        ..Default::default()
    };
    let alpha_article = alpha.get_article(get_article_data.clone()).await?;
    assert!(!alpha_article.article.pending);

    Ok(())
}
