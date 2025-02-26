use std::{fs::create_dir_all, io::Result};

/// Create site folder so include_dir macro for assets doesn't throw error in clean repo
fn main() -> Result<()> {
    create_dir_all("../../target/site/")?;
    Ok(())
}
