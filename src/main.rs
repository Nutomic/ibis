use ibis::error::MyResult;
use ibis::start;
use tracing::log::LevelFilter;

#[tokio::main]
pub async fn main() -> MyResult<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .filter_module("activitypub_federation", LevelFilter::Info)
        .filter_module("ibis", LevelFilter::Info)
        .init();
    let database_url = "postgres://ibis:password@localhost:5432/ibis";
    start("localhost:8131", database_url).await?;
    Ok(())
}
