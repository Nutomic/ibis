use activitypub_federation::config::Data;
use anyhow::anyhow;
use diffy::{apply, Patch};
use ibis_database::{
    common::{
        article::{Article, Edit, EditVersion},
        utils::http_protocol_str,
    },
    error::{BackendError, BackendResult},
    impls::IbisContext,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use url::{ParseError, Url};

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
        http_protocol_str(),
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
    edits: &Vec<Edit>,
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

pub fn can_edit_article(article: &Article, is_admin: bool) -> BackendResult<()> {
    if article.protected {
        if !article.local && !is_admin {
            return Err(BackendError(anyhow!(
                "Article is protected, only admins on origin instance can edit".to_string()
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Utc;
    use diffy::create_patch;
    use ibis_database::common::newtypes::{ArticleId, EditId, PersonId};

    fn create_edits() -> BackendResult<Vec<Edit>> {
        let generate_edit = |a, b| -> BackendResult<Edit> {
            let diff = create_patch(a, b).to_string();
            Ok(Edit {
                id: EditId(0),
                creator_id: PersonId(0),
                hash: EditVersion::new(&diff),
                ap_id: Url::parse("http://example.com")?.into(),
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
