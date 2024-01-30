#[cfg(feature = "ssr")]
#[tokio::main]
pub async fn main() -> ibis_lib::backend::error::MyResult<()> {
    use log::LevelFilter;
    env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .filter_module("activitypub_federation", LevelFilter::Info)
        .filter_module("ibis", LevelFilter::Info)
        .init();
    let database_url = std::env::var("IBIS_DATABASE_URL")
        .unwrap_or("postgres://ibis:password@localhost:5432/ibis".to_string());
    let port = std::env::var("IBIS_BACKEND_PORT").unwrap_or("8081".to_string());
    ibis_lib::backend::start(&format!("127.0.0.1:{port}"), &database_url).await?;
    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn main() {
    use ibis_lib::frontend::app::App;
    use leptos::mount_to_body;
    use leptos::view;

    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {  <App/> }
    });
}
