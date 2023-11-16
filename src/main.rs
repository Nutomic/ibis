use fediwiki::error::MyResult;
use fediwiki::start;
use tracing::log::LevelFilter;

#[tokio::main]
pub async fn main() -> MyResult<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .filter_module("activitypub_federation", LevelFilter::Info)
        .filter_module("fediwiki", LevelFilter::Info)
        .init();
    start("localhost:8131").await?;
    Ok(())
}
