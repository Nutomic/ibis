#[cfg(feature = "ssr")]
#[tokio::main]
pub async fn main() -> ibis_database::error::BackendResult<()> {
    use ibis_database::config::IbisConfig;
    use log::LevelFilter;

    if std::env::args().collect::<Vec<_>>().get(1) == Some(&"--print-config".to_string()) {
        println!("{}", doku::to_toml::<IbisConfig>());
        std::process::exit(0);
    }

    env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .filter_module("activitypub_federation", LevelFilter::Debug)
        .filter_module("ibis", LevelFilter::Debug)
        .init();

    let ibis_config = IbisConfig::read()?;
    ibis::backend::start(ibis_config, None, None).await?;
    Ok(())
}
