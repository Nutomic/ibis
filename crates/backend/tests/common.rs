#![expect(clippy::unwrap_used)]

use anyhow::Result;
use ibis_api_client::{ApiClient, user::RegisterUserParams};
use ibis_backend::start;
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
    thread::spawn,
};
use tokio::{join, sync::oneshot, task::JoinHandle};

pub struct TestData(pub IbisInstance, pub IbisInstance, pub IbisInstance);

impl TestData {
    pub async fn start(article_approval: bool) -> Self {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            env_logger::builder()
                .filter_level(LevelFilter::Warn)
                //.filter_module("activitypub_federation", LevelFilter::Info)
                //.filter_module("ibis", LevelFilter::Info)
                .init();
        });

        // Run things on different ports and db paths to allow parallel tests
        static COUNTER: AtomicI32 = AtomicI32::new(0);
        let current_run = COUNTER.fetch_add(1, Ordering::Relaxed);

        let first_port = 8100 + (current_run * 3);
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

        let (alpha, beta, gamma) = join!(
            IbisInstance::start(alpha_db_path, port_alpha, "alpha", article_approval),
            IbisInstance::start(beta_db_path, port_beta, "beta", article_approval),
            IbisInstance::start(gamma_db_path, port_gamma, "gamma", article_approval)
        );

        Self(alpha, beta, gamma)
    }

    pub fn stop(alpha: IbisInstance, beta: IbisInstance, gamma: IbisInstance) -> Result<()> {
        for j in [alpha.stop(), beta.stop(), gamma.stop()] {
            j.join().unwrap();
        }
        Ok(())
    }
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

pub struct IbisInstance {
    pub api_client: ApiClient,
    db_path: String,
    db_handle: JoinHandle<()>,
    pub hostname: String,
}

impl IbisInstance {
    fn prepare_db(db_path: String) -> std::thread::JoinHandle<()> {
        // stop any db leftover from previous run
        Self::stop_internal(db_path.clone());
        // remove old db
        remove_dir_all(&db_path).unwrap();
        spawn(move || {
            Command::new("./scripts/start_test_db.sh")
                .arg(&db_path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .output()
                .unwrap();
        })
    }

    async fn start(db_path: String, port: i32, username: &str, article_approval: bool) -> Self {
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
                article_approval,
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
            password: "hunter2".to_string(),
        };
        api_client.register(params).await.unwrap();
        Self {
            api_client,
            db_path,
            db_handle,
            hostname,
        }
    }

    fn stop(self) -> std::thread::JoinHandle<()> {
        self.db_handle.abort();
        Self::stop_internal(self.db_path)
    }

    fn stop_internal(db_path: String) -> std::thread::JoinHandle<()> {
        spawn(move || {
            Command::new("./scripts/stop_test_db.sh")
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

pub const TEST_ARTICLE_DEFAULT_TEXT: &str = "some example text\n";
