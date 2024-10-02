#[cfg(feature = "ssr")]
#[tokio::main]
pub async fn main() -> ibis_lib::backend::error::MyResult<()> {
    use ibis_lib::backend::config::IbisConfig;
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
    ibis_lib::backend::start(ibis_config).await?;
    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn main() {
    use ibis_lib::frontend::app::App;
    use leptos::{mount_to_body, view};

    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! { <App /> }
    });
}
