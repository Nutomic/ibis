use ibis_lib::backend::config::{IbisConfig, IbisConfigFederation};
use ibis_lib::backend::start;
use ibis_lib::common::RegisterUserData;
use ibis_lib::frontend::api::ApiClient;
use ibis_lib::frontend::error::MyResult;
use reqwest::ClientBuilder;
use std::env::current_dir;
use std::fs::{create_dir_all, remove_dir_all};
use std::ops::Deref;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Once;
use std::thread::{sleep, spawn};
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::log::LevelFilter;

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
    pub api_client: ApiClient,
    db_path: String,
    db_handle: JoinHandle<()>,
}

impl IbisInstance {
    fn prepare_db(db_path: String) -> std::thread::JoinHandle<()> {
        // stop any db leftover from previous run
        Self::stop_internal(db_path.clone());
        // remove old db
        remove_dir_all(&db_path).unwrap();
        spawn(move || {
            Command::new("./tests/scripts/start_dev_db.sh")
                .arg(&db_path)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()
                .unwrap();
        })
    }

    async fn start(db_path: String, port: i32, username: &str) -> Self {
        let database_url = format!("postgresql://ibis:password@/ibis?host={db_path}");
        let hostname = format!("localhost:{port}");
        let bind = format!("127.0.0.1:{port}").parse().unwrap();
        let config = IbisConfig {
            bind,
            database_url,
            registration_open: true,
            federation: IbisConfigFederation {
                domain: hostname.clone(),
                ..Default::default()
            },
            ..Default::default()
        };
        let handle = tokio::task::spawn(async move {
            start(config).await.unwrap();
        });
        // wait a moment for the backend to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        let form = RegisterUserData {
            username: username.to_string(),
            password: "hunter2".to_string(),
        };
        let client = ClientBuilder::new().cookie_store(true).build().unwrap();
        let api_client = ApiClient::new(client, hostname.clone());
        api_client.register(form).await.unwrap();
        Self {
            api_client,
            db_path,
            db_handle: handle,
        }
    }

    fn stop(self) -> std::thread::JoinHandle<()> {
        self.db_handle.abort();
        Self::stop_internal(self.db_path)
    }

    fn stop_internal(db_path: String) -> std::thread::JoinHandle<()> {
        spawn(move || {
            Command::new("./tests/scripts/stop_dev_db.sh")
                .arg(db_path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .output()
                .unwrap();
        })
    }
}

impl Deref for IbisInstance {
    type Target = ApiClient;

    fn deref(&self) -> &Self::Target {
        &self.api_client
    }
}

pub const TEST_ARTICLE_DEFAULT_TEXT: &str = "some\nexample\ntext\n";
