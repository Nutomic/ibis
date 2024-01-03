use crate::backend::error::MyResult;
use crate::common::EditVersion;
use sha2::{Digest, Sha256};
use uuid::Uuid;

impl EditVersion {
    pub fn new(diff: &str) -> MyResult<Self> {
        let mut sha256 = Sha256::new();
        sha256.update(diff);
        let hash_bytes = sha256.finalize();
        let uuid = Uuid::from_slice(&hash_bytes.as_slice()[..16])?;
        Ok(EditVersion(uuid))
    }

    pub fn hash(&self) -> String {
        hex::encode(self.0.into_bytes())
    }
}

impl Default for EditVersion {
    fn default() -> Self {
        EditVersion::new("").unwrap()
    }
}

#[test]
fn test_edit_versions() -> MyResult<()> {
    let default = EditVersion::default();
    assert_eq!("e3b0c44298fc1c149afbf4c8996fb924", default.hash());

    let version = EditVersion::new("test")?;
    assert_eq!("9f86d081884c7d659a2feaa0c55ad015", version.hash());

    Ok(())
}
