use anyhow::anyhow;
use ibis::backend::api::article::{CreateArticleData, EditArticleData, ForkArticleData};
use ibis::backend::api::instance::FollowInstance;
use ibis::backend::api::user::RegisterUserData;
use ibis::backend::api::user::{LoginResponse, LoginUserData};
use ibis::backend::api::ResolveObject;
use ibis::backend::database::conflict::ApiConflict;
use ibis::backend::database::instance::DbInstance;
use ibis::backend::error::MyResult;
use ibis::backend::start;
use ibis::common::ArticleView;
use ibis::frontend::api;
use ibis::frontend::api::get_query;
use ibis_lib::frontend::api;
use once_cell::sync::Lazy;
use reqwest::{Client, StatusCode};
use serde::de::Deserialize;
use std::env::current_dir;
use std::fs::create_dir_all;
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
    pub alpha: IbisInstance,
    pub beta: IbisInstance,
    pub gamma: IbisInstance,
}

impl TestData {
    pub async fn start() -> Self {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            env_logger::builder()
                .filter_level(LevelFilter::Warn)
                .filter_module("activitypub_federation", LevelFilter::Info)
                .filter_module("ibis", LevelFilter::Info)
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
            IbisInstance::prepare_db(alpha_db_path.clone()),
            IbisInstance::prepare_db(beta_db_path.clone()),
            IbisInstance::prepare_db(gamma_db_path.clone()),
        ] {
            j.join().unwrap();
        }

        Self {
            alpha: IbisInstance::start(alpha_db_path, port_alpha, "alpha").await,
            beta: IbisInstance::start(beta_db_path, port_beta, "beta").await,
            gamma: IbisInstance::start(gamma_db_path, port_gamma, "gamma").await,
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
    let path = format!(
        "{}/target/test_db/{name}-{port}",
        current_dir().unwrap().display()
    );
    create_dir_all(&path).unwrap();
    path
}

pub struct IbisInstance {
    pub hostname: String,
    pub jwt: String,
    db_path: String,
    db_handle: JoinHandle<()>,
}

impl IbisInstance {
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

    async fn start(db_path: String, port: i32, username: &str) -> Self {
        let db_url = format!("postgresql://lemmy:password@/lemmy?host={db_path}");
        let hostname = format!("localhost:{port}");
        let hostname_ = hostname.clone();
        let handle = tokio::task::spawn(async move {
            start(&hostname_, &db_url).await.unwrap();
        });
        // wait a moment for the backend to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        let register_res = api::register(&hostname, username, "hunter2").await.unwrap();
        assert!(!register_res.jwt.is_empty());
        Self {
            jwt: register_res.jwt,
            hostname,
            db_path,
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

pub async fn create_article(instance: &IbisInstance, title: String) -> MyResult<ArticleView> {
    let create_form = CreateArticleData {
        title: title.clone(),
    };
    let req = CLIENT
        .post(format!("http://{}/api/v1/article", &instance.hostname))
        .form(&create_form)
        .bearer_auth(&instance.jwt);
    let article: ArticleView = api::handle_json_res(req).await?;

    // create initial edit to ensure that conflicts are generated (there are no conflicts on empty file)
    let edit_form = EditArticleData {
        article_id: article.article.id,
        new_text: TEST_ARTICLE_DEFAULT_TEXT.to_string(),
        previous_version_id: article.latest_version,
        resolve_conflict_id: None,
    };
    edit_article(instance, &edit_form).await
}

pub async fn edit_article_with_conflict(
    instance: &IbisInstance,
    edit_form: &EditArticleData,
) -> MyResult<Option<ApiConflict>> {
    let req = CLIENT
        .patch(format!("http://{}/api/v1/article", instance.hostname))
        .form(edit_form)
        .bearer_auth(&instance.jwt);
    api::handle_json_res(req).await
}

pub async fn get_conflicts(instance: &IbisInstance) -> MyResult<Vec<ApiConflict>> {
    let req = CLIENT
        .get(format!(
            "http://{}/api/v1/edit_conflicts",
            &instance.hostname
        ))
        .bearer_auth(&instance.jwt);
    api::handle_json_res(req).await
}

pub async fn edit_article(
    instance: &IbisInstance,
    edit_form: &EditArticleData,
) -> MyResult<ArticleView> {
    let edit_res = edit_article_with_conflict(instance, edit_form).await?;
    assert!(edit_res.is_none());
    api::get_article(&instance.hostname, edit_form.article_id).await
}

pub async fn get<T>(hostname: &str, endpoint: &str) -> MyResult<T>
where
    T: for<'de> Deserialize<'de>,
{
    get_query(hostname, endpoint, None::<i32>).await
}

pub async fn fork_article(
    instance: &IbisInstance,
    form: &ForkArticleData,
) -> MyResult<ArticleView> {
    let req = CLIENT
        .post(format!("http://{}/api/v1/article/fork", instance.hostname))
        .form(form)
        .bearer_auth(&instance.jwt);
    api::handle_json_res(req).await
}

pub async fn follow_instance(instance: &IbisInstance, follow_instance: &str) -> MyResult<()> {
    // fetch beta instance on alpha
    let resolve_form = ResolveObject {
        id: Url::parse(&format!("http://{}", follow_instance))?,
    };
    let instance_resolved: DbInstance =
        api::get_query(&instance.hostname, "resolve_instance", Some(resolve_form)).await?;

    // send follow
    let follow_form = FollowInstance {
        id: instance_resolved.id,
    };
    // cant use post helper because follow doesnt return json
    let res = CLIENT
        .post(format!(
            "http://{}/api/v1/instance/follow",
            instance.hostname
        ))
        .form(&follow_form)
        .bearer_auth(&instance.jwt)
        .send()
        .await?;
    if res.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(anyhow!("API error: {}", res.text().await?).into())
    }
}
