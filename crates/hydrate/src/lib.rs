use leptos::wasm_bindgen;

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use ibis_frontend::app::App;
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
