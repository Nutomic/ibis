use std::{
    fs::{create_dir_all, File},
    io::Result,
    path::Path,
};

/// Create placeholders for wasm files so that `cargo check` etc work without explicitly building
/// frontend.
fn main() -> Result<()> {
    create_dir_all("assets/dist/")?;
    let js = "assets/dist/ibis.js";
    if !Path::new(js).exists() {
        File::create(js)?;
    }
    let wasm = "assets/dist/ibis_bg.wasm";
    if !Path::new(wasm).exists() {
        File::create(wasm)?;
    }
    Ok(())
}
