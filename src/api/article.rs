use crate::database::article::{ArticleView, DbArticle, DbArticleForm};
use crate::database::conflict::{ApiConflict, DbConflict, DbConflictForm};
use crate::database::edit::{DbEdit, DbEditForm};
use crate::database::instance::DbInstance;
use crate::database::user::LocalUserView;
use crate::database::version::EditVersion;
use crate::database::MyDataHandle;
use crate::error::MyResult;
use crate::federation::activities::create_article::CreateArticle;
use crate::federation::activities::submit_article_update;
use crate::utils::generate_article_version;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use axum::extract::Query;
use axum::Extension;
use axum::Form;
use axum::Json;
use axum_macros::debug_handler;
use diffy::create_patch;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateArticleData {
    pub title: String,
}

/// Create a new article with empty text, and federate it to followers.
#[debug_handler]
pub(in crate::api) async fn create_article(
    Extension(_user): Extension<LocalUserView>,
    data: Data<MyDataHandle>,
    Form(create_article): Form<CreateArticleData>,
) -> MyResult<Json<ArticleView>> {
    let local_instance = DbInstance::read_local_instance(&data.db_connection)?;
    let ap_id = ObjectId::parse(&format!(
        "http://{}:{}/article/{}",
        local_instance.ap_id.inner().domain().unwrap(),
        local_instance.ap_id.inner().port().unwrap(),
        create_article.title
    ))?;
    let form = DbArticleForm {
        title: create_article.title,
        text: String::new(),
        ap_id,
        instance_id: local_instance.id,
        local: true,
    };
    let article = DbArticle::create(&form, &data.db_connection)?;

    CreateArticle::send_to_followers(article.clone(), &data).await?;

    Ok(Json(DbArticle::read_view(article.id, &data.db_connection)?))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EditArticleData {
    /// Id of the article to edit
    pub article_id: i32,
    /// Full, new text of the article. A diff against `previous_version` is generated on the server
    /// side to handle conflicts.
    pub new_text: String,
    /// The version that this edit is based on, ie [DbArticle.latest_version] or
    /// [ApiConflict.previous_version]
    pub previous_version_id: EditVersion,
    /// If you are resolving a conflict, pass the id to delete conflict from the database
    pub resolve_conflict_id: Option<EditVersion>,
}

/// Edit an existing article (local or remote).
///
/// It gracefully handles the case where multiple users edit an article at the same time, by
/// generating diffs against the most recent common ancestor version, and resolving conflicts
/// automatically if possible. If not, an [ApiConflict] is returned which contains data for a three-
/// way-merge (similar to git). After the conflict is resolved, resubmit the edit with
/// `resolve_conflict_id` and uppdated `previous_version`.
///
/// Conflicts are stored in the database so they can be retrieved later from `/api/v3/edit_conflicts`.
#[debug_handler]
pub(in crate::api) async fn edit_article(
    Extension(_user): Extension<LocalUserView>,
    data: Data<MyDataHandle>,
    Form(edit_form): Form<EditArticleData>,
) -> MyResult<Json<Option<ApiConflict>>> {
    // resolve conflict if any
    if let Some(resolve_conflict_id) = edit_form.resolve_conflict_id {
        DbConflict::delete(resolve_conflict_id, &data.db_connection)?;
    }
    let original_article = DbArticle::read_view(edit_form.article_id, &data.db_connection)?;

    if edit_form.previous_version_id == original_article.latest_version {
        // No intermediate changes, simply submit new version
        submit_article_update(
            &data,
            edit_form.new_text.clone(),
            edit_form.previous_version_id,
            &original_article.article,
        )
        .await?;
        Ok(Json(None))
    } else {
        // There have been other changes since this edit was initiated. Get the common ancestor
        // version and generate a diff to find out what exactly has changed.
        let ancestor =
            generate_article_version(&original_article.edits, &edit_form.previous_version_id)?;
        let patch = create_patch(&ancestor, &edit_form.new_text);

        let previous_version = DbEdit::read(&edit_form.previous_version_id, &data.db_connection)?;
        let form = DbConflictForm {
            id: EditVersion::new(&patch.to_string())?,
            diff: patch.to_string(),
            article_id: original_article.article.id,
            previous_version_id: previous_version.hash,
        };
        let conflict = DbConflict::create(&form, &data.db_connection)?;
        Ok(Json(conflict.to_api_conflict(&data).await?))
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GetArticleData {
    pub article_id: i32,
}

/// Retrieve an article by ID. It must already be stored in the local database.
#[debug_handler]
pub(in crate::api) async fn get_article(
    Query(query): Query<GetArticleData>,
    data: Data<MyDataHandle>,
) -> MyResult<Json<ArticleView>> {
    Ok(Json(DbArticle::read_view(
        query.article_id,
        &data.db_connection,
    )?))
}

#[derive(Deserialize, Serialize)]
pub struct ForkArticleData {
    // TODO: could add optional param new_title so there is no problem with title collision
    //       in case local article with same title exists. however that makes it harder to discover
    //       variants of same article.
    pub article_id: i32,
}

/// Fork a remote article to local instance. This is useful if there are disagreements about
/// how an article should be edited.
#[debug_handler]
pub(in crate::api) async fn fork_article(
    data: Data<MyDataHandle>,
    Form(fork_form): Form<ForkArticleData>,
) -> MyResult<Json<ArticleView>> {
    // TODO: lots of code duplicated from create_article(), can move it into helper
    let original_article = DbArticle::read(fork_form.article_id, &data.db_connection)?;

    let local_instance = DbInstance::read_local_instance(&data.db_connection)?;
    let ap_id = ObjectId::parse(&format!(
        "http://{}:{}/article/{}",
        local_instance.ap_id.inner().domain().unwrap(),
        local_instance.ap_id.inner().port().unwrap(),
        original_article.title
    ))?;
    let form = DbArticleForm {
        title: original_article.title.clone(),
        text: original_article.text.clone(),
        ap_id,
        instance_id: local_instance.id,
        local: true,
    };
    let article = DbArticle::create(&form, &data.db_connection)?;

    // copy edits to new article
    // this could also be done in sql
    let edits = DbEdit::read_for_article(&original_article, &data.db_connection)?;
    for e in edits {
        let ap_id = DbEditForm::generate_ap_id(&article, &e.hash)?;
        let form = DbEditForm {
            ap_id,
            diff: e.diff,
            article_id: article.id,
            hash: e.hash,
            previous_version_id: e.previous_version_id,
        };
        DbEdit::create(&form, &data.db_connection)?;
    }

    CreateArticle::send_to_followers(article.clone(), &data).await?;

    Ok(Json(DbArticle::read_view(article.id, &data.db_connection)?))
}
