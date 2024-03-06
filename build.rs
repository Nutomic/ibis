use std::{
    fs::{create_dir_all, File},
    io::Result,
};

/// Create placeholders for wasm files so that `cargo check` etc work without explicitly building
/// frontend.
fn main() -> Result<()> {
    create_dir_all("assets/dist/")?;
    File::create("assets/dist/ibis.js")?;
    File::create("assets/dist/ibis_bg.wasm")?;
    Ok(())
}
