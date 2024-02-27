use crate::backend::database::article::DbArticleForm;
use crate::backend::database::conflict::{DbConflict, DbConflictForm};
use crate::backend::database::edit::DbEditForm;
use crate::backend::database::IbisData;
use crate::backend::error::MyResult;
use crate::backend::federation::activities::create_article::CreateArticle;
use crate::backend::federation::activities::submit_article_update;
use crate::backend::utils::generate_article_version;
use crate::common::utils::extract_domain;
use crate::common::utils::http_protocol_str;
use crate::common::validation::can_edit_article;
use crate::common::LocalUserView;
use crate::common::{ApiConflict, ResolveObject};
use crate::common::{ArticleView, DbArticle, DbEdit};
use crate::common::{CreateArticleData, EditArticleData, EditVersion, ForkArticleData};
use crate::common::{DbInstance, SearchArticleData};
use crate::common::{GetArticleData, ListArticlesData};
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use anyhow::anyhow;
use axum::extract::Query;
use axum::Extension;
use axum::Form;
use axum::Json;
use axum_macros::debug_handler;
use chrono::Utc;
use diffy::create_patch;

/// Create a new article with empty text, and federate it to followers.
#[debug_handler]
pub(in crate::backend::api) async fn create_article(
    Extension(user): Extension<LocalUserView>,
    data: Data<IbisData>,
    Form(create_article): Form<CreateArticleData>,
) -> MyResult<Json<ArticleView>> {
    if create_article.title.is_empty() {
        return Err(anyhow!("Title must not be empty").into());
    }

    let local_instance = DbInstance::read_local_instance(&data)?;
    let ap_id = ObjectId::parse(&format!(
        "{}://{}/article/{}",
        http_protocol_str(),
        extract_domain(&local_instance.ap_id),
        create_article.title
    ))?;
    let form = DbArticleForm {
        title: create_article.title,
        text: String::new(),
        ap_id,
        instance_id: local_instance.id,
        local: true,
    };
    let article = DbArticle::create(form, &data)?;

    let edit_data = EditArticleData {
        article_id: article.id,
        new_text: create_article.text,
        summary: create_article.summary,
        previous_version_id: article.latest_edit_version(&data)?,
        resolve_conflict_id: None,
    };
    let _ = edit_article(Extension(user), data.reset_request_count(), Form(edit_data)).await?;

    let article_view = DbArticle::read_view(article.id, &data)?;
    CreateArticle::send_to_followers(article_view.article.clone(), &data).await?;

    Ok(Json(article_view))
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
pub(in crate::backend::api) async fn edit_article(
    Extension(user): Extension<LocalUserView>,
    data: Data<IbisData>,
    Form(mut edit_form): Form<EditArticleData>,
) -> MyResult<Json<Option<ApiConflict>>> {
    // resolve conflict if any
    if let Some(resolve_conflict_id) = edit_form.resolve_conflict_id {
        DbConflict::delete(resolve_conflict_id, &data)?;
    }
    let original_article = DbArticle::read_view(edit_form.article_id, &data)?;
    if edit_form.new_text == original_article.article.text {
        return Err(anyhow!("Edit contains no changes").into());
    }
    if edit_form.summary.is_empty() {
        return Err(anyhow!("No summary given").into());
    }
    can_edit_article(&original_article.article, user.local_user.admin)?;
    // ensure trailing newline for clean diffs
    if !edit_form.new_text.ends_with('\n') {
        edit_form.new_text.push('\n');
    }

    if edit_form.previous_version_id == original_article.latest_version {
        // No intermediate changes, simply submit new version
        submit_article_update(
            edit_form.new_text.clone(),
            edit_form.summary.clone(),
            edit_form.previous_version_id,
            &original_article.article,
            user.person.id,
            &data,
        )
        .await?;
        Ok(Json(None))
    } else {
        // There have been other changes since this edit was initiated. Get the common ancestor
        // version and generate a diff to find out what exactly has changed.
        let ancestor =
            generate_article_version(&original_article.edits, &edit_form.previous_version_id)?;
        let patch = create_patch(&ancestor, &edit_form.new_text);

        let previous_version = DbEdit::read(&edit_form.previous_version_id, &data)?;
        let form = DbConflictForm {
            hash: EditVersion::new(&patch.to_string()),
            diff: patch.to_string(),
            summary: edit_form.summary.clone(),
            creator_id: user.local_user.id,
            article_id: original_article.article.id,
            previous_version_id: previous_version.hash,
        };
        let conflict = DbConflict::create(&form, &data)?;
        Ok(Json(conflict.to_api_conflict(&data).await?))
    }
}

/// Retrieve an article by ID. It must already be stored in the local database.
#[debug_handler]
pub(in crate::backend::api) async fn get_article(
    Query(query): Query<GetArticleData>,
    data: Data<IbisData>,
) -> MyResult<Json<ArticleView>> {
    match (query.title, query.id) {
        (Some(title), None) => Ok(Json(DbArticle::read_view_title(
            &title,
            query.domain,
            &data,
        )?)),
        (None, Some(id)) => {
            if query.domain.is_some() {
                return Err(anyhow!("Cant combine id and instance_domain").into());
            }
            Ok(Json(DbArticle::read_view(id, &data)?))
        }
        _ => Err(anyhow!("Must pass exactly one of title, id").into()),
    }
}

#[debug_handler]
pub(in crate::backend::api) async fn list_articles(
    Query(query): Query<ListArticlesData>,
    data: Data<IbisData>,
) -> MyResult<Json<Vec<DbArticle>>> {
    let only_local = query.only_local.unwrap_or(false);
    Ok(Json(DbArticle::read_all(only_local, &data)?))
}

/// Fork a remote article to local instance. This is useful if there are disagreements about
/// how an article should be edited.
#[debug_handler]
pub(in crate::backend::api) async fn fork_article(
    Extension(_user): Extension<LocalUserView>,
    data: Data<IbisData>,
    Form(fork_form): Form<ForkArticleData>,
) -> MyResult<Json<ArticleView>> {
    // TODO: lots of code duplicated from create_article(), can move it into helper
    let original_article = DbArticle::read(fork_form.article_id, &data)?;

    let local_instance = DbInstance::read_local_instance(&data)?;
    let ap_id = ObjectId::parse(&format!(
        "{}://{}/article/{}",
        http_protocol_str(),
        extract_domain(&local_instance.ap_id),
        &fork_form.new_title
    ))?;
    let form = DbArticleForm {
        title: fork_form.new_title,
        text: original_article.text.clone(),
        ap_id,
        instance_id: local_instance.id,
        local: true,
    };
    let article = DbArticle::create(form, &data)?;

    // copy edits to new article
    // this could also be done in sql
    let edits = DbEdit::read_for_article(&original_article, &data)?
        .into_iter()
        .map(|e| e.edit)
        .collect::<Vec<_>>();
    for e in edits {
        let ap_id = DbEditForm::generate_ap_id(&article, &e.hash)?;
        let form = DbEditForm {
            ap_id,
            diff: e.diff,
            summary: e.summary,
            creator_id: e.creator_id,
            article_id: article.id,
            hash: e.hash,
            previous_version_id: e.previous_version_id,
            created: Utc::now(),
        };
        DbEdit::create(&form, &data)?;
    }

    CreateArticle::send_to_followers(article.clone(), &data).await?;

    Ok(Json(DbArticle::read_view(article.id, &data)?))
}

/// Fetch a remote article, including edits collection. Allows viewing and editing. Note that new
/// article changes can only be received if we follow the instance, or if it is refetched manually.
#[debug_handler]
pub(super) async fn resolve_article(
    Query(query): Query<ResolveObject>,
    data: Data<IbisData>,
) -> MyResult<Json<ArticleView>> {
    let article: DbArticle = ObjectId::from(query.id).dereference(&data).await?;
    let edits = DbEdit::read_for_article(&article, &data)?;
    let latest_version = edits
        .last()
        .expect("has at least one edit")
        .edit
        .hash
        .clone();
    Ok(Json(ArticleView {
        article,
        edits,
        latest_version,
    }))
}

/// Search articles for matching title or body text.
#[debug_handler]
pub(super) async fn search_article(
    Query(query): Query<SearchArticleData>,
    data: Data<IbisData>,
) -> MyResult<Json<Vec<DbArticle>>> {
    if query.query.is_empty() {
        return Err(anyhow!("Query is empty").into());
    }
    let article = DbArticle::search(&query.query, &data)?;
    Ok(Json(article))
}
