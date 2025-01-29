use crate::{
    backend::{database::IbisContext, utils::error::BackendResult},
    common::{
        article::{DbEdit, EditVersion},
        utils,
    },
};
use activitypub_federation::{
    config::Data,
    http_signatures::{generate_actor_keypair, Keypair},
};
use anyhow::anyhow;
use diffy::{apply, Patch};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::sync::LazyLock;
use url::{ParseError, Url};

pub mod config;
pub mod error;
pub(super) mod scheduled_tasks;
pub(super) mod validate;

pub(super) fn generate_activity_id(context: &Data<IbisContext>) -> Result<Url, ParseError> {
    let domain = &context.config.federation.domain;
    let id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    Url::parse(&format!(
        "{}://{}/activity/{}",
        utils::http_protocol_str(),
        domain,
        id
    ))
}

/// Starting from empty string, apply edits until the specified version is reached. If no version is
/// given, apply all edits up to latest version.
///
/// TODO: testing
/// TODO: should cache all these generated versions
pub(super) fn generate_article_version(
    edits: &Vec<DbEdit>,
    version: &EditVersion,
) -> BackendResult<String> {
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

/// Use a single static keypair during testing which is signficantly faster than
/// generating dozens of keys from scratch.
pub fn generate_keypair() -> BackendResult<Keypair> {
    if cfg!(debug_assertions) {
        static KEYPAIR: LazyLock<Keypair> =
            LazyLock::new(|| generate_actor_keypair().expect("generate keypair"));
        Ok(KEYPAIR.clone())
    } else {
        Ok(generate_actor_keypair()?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::{
        article::DbEdit,
        newtypes::{ArticleId, EditId, PersonId},
    };
    use activitypub_federation::fetch::object_id::ObjectId;
    use chrono::Utc;
    use diffy::create_patch;

    fn create_edits() -> BackendResult<Vec<DbEdit>> {
        let generate_edit = |a, b| -> BackendResult<DbEdit> {
            let diff = create_patch(a, b).to_string();
            Ok(DbEdit {
                id: EditId(0),
                creator_id: PersonId(0),
                hash: EditVersion::new(&diff),
                ap_id: ObjectId::parse("http://example.com")?,
                diff,
                summary: String::new(),
                article_id: ArticleId(0),
                previous_version_id: Default::default(),
                published: Utc::now(),
                pending: false,
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
    fn test_generate_article_version() -> BackendResult<()> {
        let edits = create_edits()?;
        let generated = generate_article_version(&edits, &edits[1].hash)?;
        assert_eq!("sda\n", generated);
        Ok(())
    }

    #[test]
    fn test_generate_invalid_version() -> BackendResult<()> {
        let edits = create_edits()?;
        let generated = generate_article_version(&edits, &EditVersion::new("invalid"));
        assert!(generated.is_err());
        Ok(())
    }

    #[test]
    fn test_generate_first_version() -> BackendResult<()> {
        let edits = create_edits()?;
        let generated = generate_article_version(&edits, &EditVersion::default())?;
        assert_eq!("", generated);
        Ok(())
    }
}
