use crate::database::edit::DbEdit;
use crate::database::version::EditVersion;
use crate::error::MyResult;
use anyhow::anyhow;
use diffy::{apply, Patch};
use rand::{distributions::Alphanumeric, thread_rng, Rng};

use url::{ParseError, Url};

pub fn generate_activity_id(domain: &Url) -> Result<Url, ParseError> {
    let port = domain.port().unwrap();
    let domain = domain.domain().unwrap();
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
