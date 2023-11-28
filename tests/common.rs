use fediwiki::api::{
    ApiConflict, CreateArticleData, EditArticleData, FollowInstance, GetArticleData, ResolveObject,
};
use fediwiki::error::MyResult;
use fediwiki::federation::objects::article::DbArticle;
use fediwiki::federation::objects::instance::DbInstance;
use fediwiki::start;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::de::Deserialize;
use serde::ser::Serialize;
use std::sync::Once;
use tokio::task::JoinHandle;
use tracing::log::LevelFilter;
use url::Url;

pub static CLIENT: Lazy<Client> = Lazy::new(Client::new);

pub struct TestData {
    pub hostname_alpha: &'static str,
    pub hostname_beta: &'static str,
    pub hostname_gamma: &'static str,
    handle_alpha: JoinHandle<()>,
    handle_beta: JoinHandle<()>,
    handle_gamma: JoinHandle<()>,
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
        let hostname_gamma = "localhost:8133";
        let handle_alpha = tokio::task::spawn(async {
            start(hostname_alpha).await.unwrap();
        });
        let handle_beta = tokio::task::spawn(async {
            start(hostname_beta).await.unwrap();
        });
        let handle_gamma = tokio::task::spawn(async {
            start(hostname_gamma).await.unwrap();
        });
        Self {
            hostname_alpha,
            hostname_beta,
            hostname_gamma,
            handle_alpha,
            handle_beta,
            handle_gamma,
        }
    }

    pub fn stop(self) -> MyResult<()> {
        self.handle_alpha.abort();
        self.handle_beta.abort();
        self.handle_gamma.abort();
        Ok(())
    }
}

pub const TEST_ARTICLE_DEFAULT_TEXT: &str = "some\nexample\ntext\n";

pub async fn create_article(hostname: &str, title: String) -> MyResult<DbArticle> {
    let create_form = CreateArticleData {
        title: title.clone(),
    };
    let article: DbArticle = post(hostname, "article", &create_form).await?;
    // create initial edit to ensure that conflicts are generated (there are no conflicts on empty file)
    let edit_form = EditArticleData {
        ap_id: article.ap_id,
        new_text: TEST_ARTICLE_DEFAULT_TEXT.to_string(),
        previous_version: article.latest_version,
        resolve_conflict_id: None,
    };
    edit_article(hostname, &title, &edit_form).await
}

pub async fn get_article(hostname: &str, title: &str) -> MyResult<DbArticle> {
    let get_article = GetArticleData {
        title: title.to_string(),
    };
    get_query::<DbArticle, _>(hostname, "article", Some(get_article.clone())).await
}

pub async fn edit_article_with_conflict(
    hostname: &str,
    edit_form: &EditArticleData,
) -> MyResult<Option<ApiConflict>> {
    Ok(CLIENT
        .patch(format!("http://{}/api/v1/article", hostname))
        .form(edit_form)
        .send()
        .await?
        .json()
        .await?)
}

pub async fn edit_article(
    hostname: &str,
    title: &str,
    edit_form: &EditArticleData,
) -> MyResult<DbArticle> {
    let edit_res: Option<ApiConflict> = CLIENT
        .patch(format!("http://{}/api/v1/article", hostname))
        .form(edit_form)
        .send()
        .await?
        .json()
        .await?;
    assert!(edit_res.is_none());
    let get_article = GetArticleData {
        title: title.to_string(),
    };
    let updated_article: DbArticle = get_query(hostname, "article", Some(get_article)).await?;
    Ok(updated_article)
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

pub async fn follow_instance(follow_instance: &str, followed_instance: &str) -> MyResult<()> {
    // fetch beta instance on alpha
    let resolve_form = ResolveObject {
        id: Url::parse(&format!("http://{}", followed_instance))?,
    };
    let instance_resolved: DbInstance =
        get_query(followed_instance, "resolve_instance", Some(resolve_form)).await?;

    // send follow
    let follow_form = FollowInstance {
        instance_id: instance_resolved.ap_id,
    };
    // cant use post helper because follow doesnt return json
    CLIENT
        .post(format!("http://{}/api/v1/instance/follow", follow_instance))
        .form(&follow_form)
        .send()
        .await?;
    Ok(())
}
