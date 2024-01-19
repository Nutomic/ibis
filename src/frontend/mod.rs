pub mod api;
pub mod app;
mod components;
pub mod error;
pub mod pages;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    /*
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    leptos::mount_to_body(move || {
        view! {  <App/> }
    });
    info!("test 1");
     */
}
