use fediwiki::api::{
    ApiConflict, CreateArticleData, EditArticleData, FollowInstance, GetArticleData, ResolveObject,
};
use fediwiki::database::article::ArticleView;
use fediwiki::database::instance::DbInstance;
use fediwiki::error::MyResult;
use fediwiki::start;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::de::Deserialize;
use serde::ser::Serialize;
use std::env::current_dir;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Once;
use std::thread::{sleep, spawn};
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::log::LevelFilter;
use url::Url;

pub static CLIENT: Lazy<Client> = Lazy::new(Client::new);

pub struct TestData {
    pub alpha: FediwikiInstance,
    pub beta: FediwikiInstance,
    pub gamma: FediwikiInstance,
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

        // Run things on different ports and db paths to allow parallel tests
        static COUNTER: AtomicI32 = AtomicI32::new(0);
        let current_run = COUNTER.fetch_add(1, Ordering::Relaxed);

        // Give each test a moment to start its postgres databases
        sleep(Duration::from_millis(current_run as u64 * 500));

        let first_port = 8000 + (current_run * 3);
        let port_alpha = first_port;
        let port_beta = first_port + 1;
        let port_gamma = first_port + 2;

        let alpha_db_path = generate_db_path("alpha", port_alpha);
        let beta_db_path = generate_db_path("beta", port_beta);
        let gamma_db_path = generate_db_path("gamma", port_gamma);

        // initialize postgres databases in parallel because its slow
        for j in [
            FediwikiInstance::prepare_db(alpha_db_path.clone()),
            FediwikiInstance::prepare_db(beta_db_path.clone()),
            FediwikiInstance::prepare_db(gamma_db_path.clone()),
        ] {
            j.join().unwrap();
        }

        Self {
            alpha: FediwikiInstance::start(alpha_db_path, port_alpha),
            beta: FediwikiInstance::start(beta_db_path, port_beta),
            gamma: FediwikiInstance::start(gamma_db_path, port_gamma),
        }
    }

    pub fn stop(self) -> MyResult<()> {
        for j in [self.alpha.stop(), self.beta.stop(), self.gamma.stop()] {
            j.join().unwrap();
        }
        Ok(())
    }
}

/// Generate a unique db path for each postgres so that tests can run in parallel.
fn generate_db_path(name: &'static str, port: i32) -> String {
    format!(
        "{}/target/test_db/{name}-{port}",
        current_dir().unwrap().display()
    )
}

pub struct FediwikiInstance {
    pub hostname: String,
    db_path: String,
    db_handle: JoinHandle<()>,
}

impl FediwikiInstance {
    fn prepare_db(db_path: String) -> std::thread::JoinHandle<()> {
        spawn(move || {
            Command::new("./tests/scripts/start_dev_db.sh")
                .arg(&db_path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .output()
                .unwrap();
        })
    }

    fn start(db_path: String, port: i32) -> Self {
        let db_url = format!("postgresql://lemmy:password@/lemmy?host={db_path}");
        let hostname = format!("localhost:{port}");
        let hostname_ = hostname.clone();
        let handle = tokio::task::spawn(async move {
            start(&hostname_, &db_url).await.unwrap();
        });
        Self {
            db_path,
            hostname,
            db_handle: handle,
        }
    }

    fn stop(self) -> std::thread::JoinHandle<()> {
        self.db_handle.abort();
        spawn(move || {
            Command::new("./tests/scripts/stop_dev_db.sh")
                .arg(&self.db_path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .output()
                .unwrap();
        })
    }
}

pub const TEST_ARTICLE_DEFAULT_TEXT: &str = "some\nexample\ntext\n";

pub async fn create_article(hostname: &str, title: String) -> MyResult<ArticleView> {
    let create_form = CreateArticleData {
        title: title.clone(),
    };
    let article: ArticleView = post(hostname, "article", &create_form).await?;
    // create initial edit to ensure that conflicts are generated (there are no conflicts on empty file)
    let edit_form = EditArticleData {
        article_id: article.article.id,
        new_text: TEST_ARTICLE_DEFAULT_TEXT.to_string(),
        previous_version: article.latest_version,
        resolve_conflict_id: None,
    };
    edit_article(hostname, &edit_form).await
}

pub async fn get_article(hostname: &str, article_id: i32) -> MyResult<ArticleView> {
    let get_article = GetArticleData { article_id };
    get_query::<ArticleView, _>(hostname, "article", Some(get_article.clone())).await
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

pub async fn edit_article(hostname: &str, edit_form: &EditArticleData) -> MyResult<ArticleView> {
    let edit_res: Option<ApiConflict> = CLIENT
        .patch(format!("http://{}/api/v1/article", hostname))
        .form(&edit_form)
        .send()
        .await?
        .json()
        .await?;
    assert!(edit_res.is_none());
    get_article(hostname, edit_form.article_id).await
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

pub async fn post<T: Serialize, R>(hostname: &str, endpoint: &str, form: &T) -> MyResult<R>
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
        id: instance_resolved.id,
    };
    // cant use post helper because follow doesnt return json
    CLIENT
        .post(format!("http://{}/api/v1/instance/follow", follow_instance))
        .form(&follow_form)
        .send()
        .await?;
    Ok(())
}
