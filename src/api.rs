use crate::database::article::{DbArticle, DbArticleForm};
use crate::database::edit::{DbEdit, EditVersion};
use crate::database::{DbConflict, MyDataHandle};
use crate::error::MyResult;
use crate::federation::activities::create_article::CreateArticle;
use crate::federation::activities::submit_article_update;
use crate::federation::objects::instance::DbInstance;
use crate::utils::generate_article_version;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use anyhow::anyhow;
use axum::extract::Query;
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use axum_macros::debug_handler;
use diffy::create_patch;
use futures::future::try_join_all;
use rand::random;
use serde::{Deserialize, Serialize};
use url::Url;

pub fn api_routes() -> Router {
    Router::new()
        .route(
            "/article",
            get(get_article).post(create_article).patch(edit_article),
        )
        .route("/article/fork", post(fork_article))
        .route("/edit_conflicts", get(edit_conflicts))
        .route("/resolve_instance", get(resolve_instance))
        .route("/resolve_article", get(resolve_article))
        .route("/instance", get(get_local_instance))
        .route("/instance/follow", post(follow_instance))
        .route("/search", get(search_article))
}

#[derive(Deserialize, Serialize)]
pub struct CreateArticleData {
    pub title: String,
}

/// Create a new article with empty text, and federate it to followers.
#[debug_handler]
async fn create_article(
    data: Data<MyDataHandle>,
    Form(create_article): Form<CreateArticleData>,
) -> MyResult<Json<DbArticle>> {
    {
        let articles = data.articles.lock().unwrap();
        let title_exists = articles
            .iter()
            .any(|a| a.1.local && a.1.title == create_article.title);
        if title_exists {
            return Err(anyhow!("A local article with this title already exists").into());
        }
    }

    let instance_id = data.local_instance().ap_id;
    let ap_id = ObjectId::parse(&format!(
        "http://{}:{}/article/{}",
        instance_id.inner().domain().unwrap(),
        instance_id.inner().port().unwrap(),
        create_article.title
    ))?
    .into();
    let form = DbArticleForm {
        title: create_article.title,
        text: String::new(),
        ap_id,
        latest_version: Default::default(),
        instance_id,
        local: true,
    };
    let article = DbArticle::create(&form, &data.db_connection)?;

    CreateArticle::send_to_followers(article.clone(), &data).await?;

    Ok(Json(article))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EditArticleData {
    /// Id of the article to edit
    pub ap_id: ObjectId<DbArticle>,
    /// Full, new text of the article. A diff against `previous_version` is generated on the server
    /// side to handle conflicts.
    pub new_text: String,
    /// The version that this edit is based on, ie [DbArticle.latest_version] or
    /// [ApiConflict.previous_version]
    pub previous_version: EditVersion,
    /// If you are resolving a conflict, pass the id to delete conflict from the database
    pub resolve_conflict_id: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ApiConflict {
    pub id: i32,
    pub three_way_merge: String,
    pub article_id: ObjectId<DbArticle>,
    pub previous_version: EditVersion,
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
async fn edit_article(
    data: Data<MyDataHandle>,
    Form(edit_form): Form<EditArticleData>,
) -> MyResult<Json<Option<ApiConflict>>> {
    // resolve conflict if any
    if let Some(resolve_conflict_id) = &edit_form.resolve_conflict_id {
        let mut lock = data.conflicts.lock().unwrap();
        if !lock.iter().any(|c| &c.id == resolve_conflict_id) {
            return Err(anyhow!("invalid resolve conflict"))?;
        }
        lock.retain(|c| &c.id != resolve_conflict_id);
    }
    let original_article = {
        let lock = data.articles.lock().unwrap();
        let article = lock.get(edit_form.ap_id.inner()).unwrap();
        article.clone()
    };

    if edit_form.previous_version == original_article.latest_version {
        // No intermediate changes, simply submit new version
        submit_article_update(&data, edit_form.new_text.clone(), &original_article).await?;
        Ok(Json(None))
    } else {
        // There have been other changes since this edit was initiated. Get the common ancestor
        // version and generate a diff to find out what exactly has changed.
        let edits = DbEdit::for_article(original_article.id, &data.db_connection)?;
        let ancestor = generate_article_version(&edits, &edit_form.previous_version)?;
        let patch = create_patch(&ancestor, &edit_form.new_text);

        let db_conflict = DbConflict {
            id: random(),
            diff: patch.to_string(),
            article_id: original_article.ap_id.clone().into(),
            previous_version: edit_form.previous_version,
        };
        {
            let mut lock = data.conflicts.lock().unwrap();
            lock.push(db_conflict.clone());
        }
        Ok(Json(db_conflict.to_api_conflict(&data).await?))
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GetArticleData {
    pub id: i32,
}

/// Retrieve an article by ID. It must already be stored in the local database.
#[debug_handler]
async fn get_article(
    Query(query): Query<GetArticleData>,
    data: Data<MyDataHandle>,
) -> MyResult<Json<DbArticle>> {
    Ok(Json(DbArticle::read(query.id, &data.db_connection)?))
}

#[derive(Deserialize, Serialize)]
pub struct ResolveObject {
    pub id: Url,
}

/// Fetch a remote instance actor. This automatically synchronizes the remote articles collection to
/// the local instance, and allows for interactions such as following.
#[debug_handler]
async fn resolve_instance(
    Query(query): Query<ResolveObject>,
    data: Data<MyDataHandle>,
) -> MyResult<Json<DbInstance>> {
    let instance: DbInstance = ObjectId::from(query.id).dereference(&data).await?;
    Ok(Json(instance))
}

/// Fetch a remote article, including edits collection. Allows viewing and editing. Note that new
/// article changes can only be received if we follow the instance, or if it is refetched manually.
#[debug_handler]
async fn resolve_article(
    Query(query): Query<ResolveObject>,
    data: Data<MyDataHandle>,
) -> MyResult<Json<DbArticle>> {
    let article: DbArticle = ObjectId::from(query.id).dereference(&data).await?;
    Ok(Json(article))
}

/// Retrieve the local instance info.
#[debug_handler]
async fn get_local_instance(data: Data<MyDataHandle>) -> MyResult<Json<DbInstance>> {
    Ok(Json(data.local_instance()))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FollowInstance {
    pub instance_id: ObjectId<DbInstance>,
}

/// Make the local instance follow a given remote instance, to receive activities about new and
/// updated articles.
#[debug_handler]
async fn follow_instance(
    data: Data<MyDataHandle>,
    Form(query): Form<FollowInstance>,
) -> MyResult<()> {
    let instance = query.instance_id.dereference(&data).await?;
    data.local_instance().follow(&instance, &data).await?;
    Ok(())
}

/// Get a list of all unresolved edit conflicts.
#[debug_handler]
async fn edit_conflicts(data: Data<MyDataHandle>) -> MyResult<Json<Vec<ApiConflict>>> {
    let conflicts = { data.conflicts.lock().unwrap().to_vec() };
    let conflicts: Vec<ApiConflict> = try_join_all(conflicts.into_iter().map(|c| {
        let data = data.reset_request_count();
        async move { c.to_api_conflict(&data).await }
    }))
    .await?
    .into_iter()
    .flatten()
    .collect();
    Ok(Json(conflicts))
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SearchArticleData {
    pub title: String,
}

/// Search articles by title. For now only checks exact match.
///
/// Later include partial title match and body search.
#[debug_handler]
async fn search_article(
    Query(query): Query<SearchArticleData>,
    data: Data<MyDataHandle>,
) -> MyResult<Json<Vec<DbArticle>>> {
    let articles = data.articles.lock().unwrap();
    let article = articles
        .iter()
        .filter(|a| a.1.title == query.title)
        .map(|a| a.1)
        .cloned()
        .collect();
    Ok(Json(article))
}

#[derive(Deserialize, Serialize)]
pub struct ForkArticleData {
    // TODO: could add optional param new_title so there is no problem with title collision
    //       in case local article with same title exists. however that makes it harder to discover
    //       variants of same article.
    pub ap_id: ObjectId<DbArticle>,
}

/// Fork a remote article to local instance. This is useful if there are disagreements about
/// how an article should be edited.
#[debug_handler]
async fn fork_article(
    data: Data<MyDataHandle>,
    Form(fork_form): Form<ForkArticleData>,
) -> MyResult<Json<DbArticle>> {
    let article = {
        let lock = data.articles.lock().unwrap();
        let article = lock.get(fork_form.ap_id.inner()).unwrap();
        article.clone()
    };
    if article.local {
        return Err(anyhow!("Cannot fork local article because there cant be multiple local articles with same title").into());
    }

    let original_article = {
        let lock = data.articles.lock().unwrap();
        lock.get(fork_form.ap_id.inner())
            .expect("article exists")
            .clone()
    };

    let instance_id = data.local_instance().ap_id;
    let ap_id = ObjectId::parse(&format!(
        "http://{}:{}/article/{}",
        instance_id.inner().domain().unwrap(),
        instance_id.inner().port().unwrap(),
        original_article.title
    ))?
    .into();
    let form = DbArticleForm {
        title: original_article.title.clone(),
        text: original_article.text.clone(),
        ap_id,
        latest_version: original_article.latest_version.0.clone(),
        instance_id,
        local: true,
    };
    let article = DbArticle::create(&form, &data.db_connection)?;

    // TODO: need to copy edits separately with db query

    CreateArticle::send_to_followers(article.clone(), &data).await?;

    Ok(Json(article))
}
