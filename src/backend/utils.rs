use crate::{
    backend::error::MyResult,
    common::{utils, utils::extract_domain, EditVersion, EditView},
};
use activitypub_federation::{fetch::object_id::ObjectId, traits::Object};
use anyhow::anyhow;
use diffy::{apply, Patch};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Deserialize;
use url::{ParseError, Url};

pub fn generate_activity_id<T>(for_url: &ObjectId<T>) -> Result<Url, ParseError>
where
    T: Object + Send + 'static,
    for<'de2> <T as Object>::Kind: Deserialize<'de2>,
{
    let domain = extract_domain(for_url);
    let id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    Url::parse(&format!(
        "{}://{}/objects/{}",
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
pub fn generate_article_version(edits: &Vec<EditView>, version: &EditVersion) -> MyResult<String> {
    let mut generated = String::new();
    if version == &EditVersion::default() {
        return Ok(generated);
    }
    for e in edits {
        let patch = Patch::from_str(&e.edit.diff)?;
        generated = apply(&generated, &patch)?;
        if &e.edit.hash == version {
            return Ok(generated);
        }
    }
    Err(anyhow!("failed to generate article version").into())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::{DbEdit, DbPerson};
    use activitypub_federation::fetch::object_id::ObjectId;
    use chrono::Utc;
    use diffy::create_patch;

    fn create_edits() -> MyResult<Vec<EditView>> {
        let generate_edit = |a, b| -> MyResult<EditView> {
            let diff = create_patch(a, b).to_string();
            Ok(EditView {
                edit: DbEdit {
                    id: 0,
                    creator_id: 0,
                    hash: EditVersion::new(&diff),
                    ap_id: ObjectId::parse("http://example.com")?,
                    diff,
                    summary: String::new(),
                    article_id: 0,
                    previous_version_id: Default::default(),
                    created: Utc::now(),
                },
                creator: DbPerson {
                    id: 0,
                    username: "".to_string(),
                    ap_id: ObjectId::parse("http://example.com")?,
                    inbox_url: "".to_string(),
                    public_key: "".to_string(),
                    private_key: None,
                    last_refreshed_at: Default::default(),
                    local: false,
                },
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
        let generated = generate_article_version(&edits, &edits[1].edit.hash)?;
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
