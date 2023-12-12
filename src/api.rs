use crate::database::article::{ArticleView, DbArticle, DbArticleForm};
use crate::database::conflict::{ApiConflict, DbConflict, DbConflictForm};
use crate::database::edit::{DbEdit, DbEditForm};
use crate::database::instance::{DbInstance, InstanceView};
use crate::database::user::{DbLocalUser, DbPerson};
use crate::database::version::EditVersion;
use crate::database::MyDataHandle;
use crate::error::MyResult;
use crate::federation::activities::create_article::CreateArticle;
use crate::federation::activities::follow::Follow;
use crate::federation::activities::submit_article_update;
use crate::utils::generate_article_version;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use anyhow::anyhow;
use axum::extract::Query;
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use axum_macros::debug_handler;
use bcrypt::verify;
use chrono::Utc;
use diffy::create_patch;
use futures::future::try_join_all;
use jsonwebtoken::{encode, EncodingKey, Header};
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
        .route("/user/register", post(register_user))
        .route("/user/login", post(login_user))
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
async fn edit_article(
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
async fn get_article(
    Query(query): Query<GetArticleData>,
    data: Data<MyDataHandle>,
) -> MyResult<Json<ArticleView>> {
    Ok(Json(DbArticle::read_view(
        query.article_id,
        &data.db_connection,
    )?))
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
) -> MyResult<Json<ArticleView>> {
    let article: DbArticle = ObjectId::from(query.id).dereference(&data).await?;
    let edits = DbEdit::read_for_article(&article, &data.db_connection)?;
    let latest_version = edits.last().unwrap().hash.clone();
    Ok(Json(ArticleView {
        article,
        edits,
        latest_version,
    }))
}

/// Retrieve the local instance info.
#[debug_handler]
async fn get_local_instance(data: Data<MyDataHandle>) -> MyResult<Json<InstanceView>> {
    let local_instance = DbInstance::read_local_view(&data.db_connection)?;
    Ok(Json(local_instance))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FollowInstance {
    pub id: i32,
}

/// Make the local instance follow a given remote instance, to receive activities about new and
/// updated articles.
#[debug_handler]
async fn follow_instance(
    data: Data<MyDataHandle>,
    Form(query): Form<FollowInstance>,
) -> MyResult<()> {
    let local_instance = DbInstance::read_local_instance(&data.db_connection)?;
    let target = DbInstance::read(query.id, &data.db_connection)?;
    let pending = !target.local;
    DbInstance::follow(local_instance.id, target.id, pending, &data)?;
    let instance = DbInstance::read(query.id, &data.db_connection)?;
    Follow::send(local_instance, instance, &data).await?;
    Ok(())
}

/// Get a list of all unresolved edit conflicts.
#[debug_handler]
async fn edit_conflicts(data: Data<MyDataHandle>) -> MyResult<Json<Vec<ApiConflict>>> {
    let conflicts = DbConflict::list(&data.db_connection)?;
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
    pub query: String,
}

/// Search articles for matching title or body text.
#[debug_handler]
async fn search_article(
    Query(query): Query<SearchArticleData>,
    data: Data<MyDataHandle>,
) -> MyResult<Json<Vec<DbArticle>>> {
    let article = DbArticle::search(&query.query, &data.db_connection)?;
    Ok(Json(article))
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
async fn fork_article(
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// local_user.id
    pub sub: String,
    /// hostname
    pub iss: String,
    /// Creation time as unix timestamp
    pub iat: i64,
}

pub fn generate_login_token(
    local_user: DbLocalUser,
    data: &Data<MyDataHandle>,
) -> MyResult<LoginResponse> {
    let hostname = data.domain().to_string();
    let claims = Claims {
        sub: local_user.id.to_string(),
        iss: hostname,
        iat: Utc::now().timestamp(),
    };

    // TODO: move to config
    let key = EncodingKey::from_secret("secret".as_bytes());
    let jwt = encode(&Header::default(), &claims, &key)?;
    Ok(LoginResponse { jwt })
}

#[derive(Deserialize, Serialize)]
pub struct RegisterUserData {
    pub name: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct LoginResponse {
    pub jwt: String,
}

#[debug_handler]
async fn register_user(
    data: Data<MyDataHandle>,
    Form(form): Form<RegisterUserData>,
) -> MyResult<Json<LoginResponse>> {
    let user = DbPerson::create_local(form.name, form.password, &data)?;
    Ok(Json(generate_login_token(user.local_user, &data)?))
}

#[derive(Deserialize, Serialize)]
pub struct LoginUserData {
    name: String,
    password: String,
}

#[debug_handler]
async fn login_user(
    data: Data<MyDataHandle>,
    Form(form): Form<LoginUserData>,
) -> MyResult<Json<LoginResponse>> {
    let user = DbPerson::read_local_from_name(&form.name, &data)?;
    let valid = verify(&form.password, &user.local_user.password_encrypted)?;
    if !valid {
        return Err(anyhow!("Invalid login").into());
    }
    Ok(Json(generate_login_token(user.local_user, &data)?))
}
