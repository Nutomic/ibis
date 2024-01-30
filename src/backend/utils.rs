use crate::backend::error::MyResult;
use crate::common::DbEdit;
use crate::common::EditVersion;
use anyhow::anyhow;
use diffy::{apply, Patch};
use rand::{distributions::Alphanumeric, thread_rng, Rng};

use url::{ParseError, Url};

pub fn generate_activity_id(domain: &Url) -> Result<Url, ParseError> {
    let port = domain.port().unwrap();
    let domain = domain.host_str().unwrap();
    let id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    Url::parse(&format!("http://{}:{}/objects/{}", domain, port, id))
}

/// Starting from empty string, apply edits until the specified version is reached. If no version is
/// given, apply all edits up to latest version.
///
/// TODO: testing
/// TODO: should cache all these generated versions
pub fn generate_article_version(edits: &Vec<DbEdit>, version: &EditVersion) -> MyResult<String> {
    let mut generated = String::new();
    if version == &EditVersion::default() {
        return Ok(generated);
    }
    for e in edits {
        let patch = Patch::from_str(&e.diff)?;
        generated = apply(&generated, &patch)?;
        if &e.hash == version {
            return Ok(generated);
        }
    }
    Err(anyhow!("failed to generate article version").into())
}

#[cfg(test)]
mod test {
    use super::*;
    use activitypub_federation::fetch::object_id::ObjectId;
    use chrono::Utc;
    use diffy::create_patch;

    fn create_edits() -> MyResult<Vec<DbEdit>> {
        let generate_edit = |a, b| -> MyResult<DbEdit> {
            let diff = create_patch(a, b).to_string();
            Ok(DbEdit {
                id: 0,
                creator_id: 0,
                hash: EditVersion::new(&diff),
                ap_id: ObjectId::parse("http://example.com")?,
                diff,
                summary: String::new(),
                article_id: 0,
                previous_version_id: Default::default(),
                created: Utc::now(),
            })
        };
        Ok([
            generate_edit("", "test\n")?,
            generate_edit("test\n", "sda\n")?,
            generate_edit("sda\n", "123\n")?,
        ]
        .to_vec())
    }

    #[test]
    fn test_generate_article_version() -> MyResult<()> {
        let edits = create_edits()?;
        let generated = generate_article_version(&edits, &edits[1].hash)?;
        assert_eq!("sda\n", generated);
        Ok(())
    }

    #[test]
    fn test_generate_invalid_version() -> MyResult<()> {
        let edits = create_edits()?;
        let generated = generate_article_version(&edits, &EditVersion::new("invalid"));
        assert!(generated.is_err());
        Ok(())
    }

    #[test]
    fn test_generate_first_version() -> MyResult<()> {
        let edits = create_edits()?;
        let generated = generate_article_version(&edits, &EditVersion::default())?;
        assert_eq!("", generated);
        Ok(())
    }
}
