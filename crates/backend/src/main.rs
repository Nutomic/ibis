use env_logger::Env;
use ibis::start;
use ibis_database::config::IbisConfig;

#[tokio::main]
pub async fn main() -> ibis_database::error::BackendResult<()> {
    if std::env::args().collect::<Vec<_>>().get(1) == Some(&"--print-config".to_string()) {
        println!("{}", doku::to_toml::<IbisConfig>());
        std::process::exit(0);
    }

    env_logger::Builder::from_env(
        Env::default().default_filter_or("warn,ibis=info,activitypub_federation=info"),
    )
    .init();

    let ibis_config = IbisConfig::read()?;
    start(ibis_config, None, None).await?;
    Ok(())
}
