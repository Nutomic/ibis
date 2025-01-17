use crate::{backend::utils::error::MyResult, common::instance::DbInstance};
use activitypub_federation::kinds::public;
use url::Url;

pub mod create_or_update_comment;
pub mod delete_comment;
pub mod undo_delete_comment;

/// Parameter is the return value from DbInstance::read_for_comment() for this comment.
fn generate_comment_activity_to(instance: &DbInstance) -> MyResult<Vec<Url>> {
    let followers_url = format!("{}/followers", &instance.ap_id);
    Ok(vec![public(), followers_url.parse()?])
}
