pub mod api;
pub mod app;
mod components;
pub mod error;
pub mod pages;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {}
