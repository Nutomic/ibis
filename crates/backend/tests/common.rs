#![expect(clippy::unwrap_used)]

use ibis::start;
use ibis_api_client::{ApiClient, user::RegisterUserParams};
use ibis_database::{
    common::instance::Options,
    config::{IbisConfig, IbisConfigDatabase, IbisConfigFederation},
};
use log::LevelFilter;
use std::{
    env::current_dir,
    fs::{create_dir_all, remove_dir_all},
    ops::Deref,
    process::{Command, Stdio},
    sync::{
        Once,
        atomic::{AtomicI32, Ordering},
    },
    time::Duration,
};
use test_context::AsyncTestContext;
use tokio::{
    join,
    sync::oneshot,
    task::{JoinHandle, spawn_blocking},
    time::sleep,
};

pub struct TestData(pub IbisInstance, pub IbisInstance, pub IbisInstance);

static ACTIVE: AtomicI32 = AtomicI32::new(0);

impl AsyncTestContext for TestData {
    async fn setup() -> Self {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            env_logger::builder()
                .filter_level(LevelFilter::Warn)
                //.filter_module("activitypub_federation", LevelFilter::Info)
                //.filter_module("ibis", LevelFilter::Info)
                .init();
        });

        // Limit number of concurrent tests, otherwise it can throw errors about too many open files
        let max_parallelism = std::env::var("IBIS_TEST_PARALLELISM")
            .map(|e| e.parse().unwrap())
            .unwrap_or(10);
        loop {
            let res = ACTIVE.fetch_update(Ordering::AcqRel, Ordering::Acquire, |x| {
                if x < max_parallelism {
                    Some(x + 1)
                } else {
                    None
                }
            });
            if res.is_err() {
                sleep(Duration::from_secs(1)).await;
            } else {
                break;
            }
        }

        // Run things on different ports and db paths to allow parallel tests
        static COUNTER: AtomicI32 = AtomicI32::new(0);
        let current_run = COUNTER.fetch_add(1, Ordering::Relaxed);

        let first_port = 8100 + (current_run * 3);

        let (alpha, beta, gamma) = join!(
            IbisInstance::new("alpha", first_port,),
            IbisInstance::new("beta", first_port + 1,),
            IbisInstance::new("gamma", first_port + 2,)
        );

        Self(alpha, beta, gamma)
    }

    async fn teardown(self) {
        join!(self.0.stop(), self.1.stop(), self.2.stop());
        ACTIVE.fetch_sub(1, Ordering::AcqRel);
    }
}

pub struct IbisInstance {
    pub api_client: ApiClient,
    db_path: String,
    db_handle: JoinHandle<()>,
    pub hostname: String,
}

impl IbisInstance {
    async fn new(name: &'static str, port: i32) -> Self {
        let db_path = Self::generate_db_path(name, port);
        Self::prepare_db(db_path.clone()).await;
        Self::start(db_path, port, name).await
    }

    /// Generate a unique db path for each postgres so that tests can run in parallel.
    fn generate_db_path(name: &'static str, port: i32) -> String {
        let path = format!(
            "{}/../../target/test_db/{name}-{port}",
            current_dir().unwrap().display()
        );
        create_dir_all(&path).unwrap();
        path
    }

    async fn prepare_db(db_path: String) {
        // stop any db leftover from previous run
        Self::stop_internal(db_path.clone()).await;
        spawn_blocking(move || {
            Command::new("./scripts/start_test_db.sh")
                .arg(&db_path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .output()
                .unwrap();
        })
        .await
        .unwrap();
    }

    async fn start(db_path: String, port: i32, username: &str) -> Self {
        let connection_url = format!("postgresql://ibis:password@/ibis?host={db_path}");

        let hostname = format!("localhost:{port}");
        let config = IbisConfig {
            database: IbisConfigDatabase {
                connection_url,
                ..Default::default()
            },
            federation: IbisConfigFederation {
                domain: hostname.clone(),
                ..Default::default()
            },
            options: Options {
                registration_open: true,
                email_required: false,
            },
            ..Default::default()
        };
        let api_client = ApiClient::new(Some(hostname.clone()));
        let (tx, rx) = oneshot::channel::<()>();
        let db_handle = tokio::task::spawn(async move {
            let hostname = format!("127.0.0.1:{port}");
            start(config, Some(hostname.parse().unwrap()), Some(tx))
                .await
                .unwrap();
        });
        // wait for the backend to start
        rx.await.unwrap();
        let params = RegisterUserParams {
            username: username.to_string(),
            password: "hunter22".to_string(),
            email: None,
            confirm_password: "hunter22".to_string(),
        };
        api_client.register(params).await.unwrap();
        Self {
            api_client,
            db_path,
            db_handle,
            hostname,
        }
    }

    async fn stop(self) {
        self.db_handle.abort();
        Self::stop_internal(self.db_path).await;
    }

    async fn stop_internal(db_path: String) {
        spawn_blocking(move || {
            Command::new("./scripts/stop_test_db.sh")
                .arg(&db_path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .output()
                .unwrap();
            // remove db files
            remove_dir_all(&db_path).ok();
        })
        .await
        .unwrap();
    }
}

impl Deref for IbisInstance {
    type Target = ApiClient;

    fn deref(&self) -> &Self::Target {
        &self.api_client
    }
}

pub const TEST_ARTICLE_DEFAULT_TEXT: &str = "some example text\n";
