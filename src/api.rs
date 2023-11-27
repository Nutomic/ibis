use crate::database::DatabaseHandle;
use crate::error::{Error, MyResult};
use crate::federation::activities::create_article::CreateArticle;
use crate::federation::activities::update_local_article::UpdateLocalArticle;
use crate::federation::activities::update_remote_article::UpdateRemoteArticle;
use crate::federation::objects::article::DbArticle;
use crate::federation::objects::edit::{DbEdit, EditVersion};
use crate::federation::objects::instance::DbInstance;
use crate::utils::generate_article_version;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use anyhow::anyhow;
use axum::extract::Query;
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use axum_macros::debug_handler;
use diffy::{apply, create_patch, merge};
use rand::random;
use serde::{Deserialize, Serialize};
use url::Url;

pub fn api_routes() -> Router {
    Router::new()
        .route(
            "/article",
            get(get_article).post(create_article).patch(edit_article),
        )
        .route("/edit_conflicts", get(edit_conflicts))
        .route("/resolve_instance", get(resolve_instance))
        .route("/resolve_article", get(resolve_article))
        .route("/instance", get(get_local_instance))
        .route("/instance/follow", post(follow_instance))
}

#[derive(Deserialize, Serialize)]
pub struct CreateArticleData {
    pub title: String,
}

#[debug_handler]
async fn create_article(
    data: Data<DatabaseHandle>,
    Form(create_article): Form<CreateArticleData>,
) -> MyResult<Json<DbArticle>> {
    let local_instance_id = data.local_instance().ap_id;
    let ap_id = ObjectId::parse(&format!(
        "http://{}:{}/article/{}",
        local_instance_id.inner().domain().unwrap(),
        local_instance_id.inner().port().unwrap(),
        create_article.title
    ))?;
    let article = DbArticle {
        title: create_article.title,
        text: String::new(),
        ap_id,
        latest_version: EditVersion::default(),
        edits: vec![],
        instance: local_instance_id,
        local: true,
    };
    {
        let mut articles = data.articles.lock().unwrap();
        articles.insert(article.ap_id.inner().clone(), article.clone());
    }

    CreateArticle::send_to_followers(article.clone(), &data).await?;

    Ok(Json(article))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EditArticleData {
    pub ap_id: ObjectId<DbArticle>,
    pub new_text: String,
    pub previous_version: EditVersion,
    pub resolve_conflict_id: Option<i32>,
}

// TODO: how to store conflict in db? with three-way-merge doesnt
//       necessarily make sense (might be outdated)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Conflict {
    pub id: i32,
    pub three_way_merge: String,
    pub article_id: ObjectId<DbArticle>,
    pub latest_version: EditVersion,
}

#[debug_handler]
async fn edit_article(
    data: Data<DatabaseHandle>,
    Form(edit_form): Form<EditArticleData>,
) -> MyResult<Json<Option<Conflict>>> {
    // resolve conflict if any
    if let Some(resolve_conflict_id) = &edit_form.resolve_conflict_id {
        let mut lock = data.conflicts.lock().unwrap();
        if lock.iter().find(|c| &c.id == resolve_conflict_id).is_none() {
            return Err(anyhow!("invalid resolve conflict"))?;
        }
        lock.retain(|c| &c.id != resolve_conflict_id);
    }
    let original_article = {
        let mut lock = data.articles.lock().unwrap();
        let article = lock.get_mut(edit_form.ap_id.inner()).unwrap();
        article.clone()
    };

    if edit_form.previous_version == original_article.latest_version {
        // no intermediate changes, simply submit new version
        submit_article_update(&data, &edit_form, &original_article).await?;
        Ok(Json(None))
    } else {
        // create a patch from the differences of previous version and new version
        let ancestor =
            generate_article_version(&original_article.edits, Some(&edit_form.previous_version))?;
        let patch = create_patch(&ancestor, &edit_form.new_text);
        if apply(&original_article.text, &patch).is_ok() {
            // patch applies cleanly so we are done
            submit_article_update(&data, &edit_form, &original_article).await?;
            Ok(Json(None))
        } else {
            // there is a merge conflict, do three-way-merge
            let merge = merge(&ancestor, &edit_form.new_text, &original_article.text)
                .err()
                .unwrap();

            let conflict = Conflict {
                id: random(),
                three_way_merge: merge,
                article_id: original_article.ap_id,
                latest_version: original_article.latest_version,
            };
            let mut lock = data.conflicts.lock().unwrap();
            lock.push(conflict.clone());
            return Ok(Json(Some(conflict)));
        }
    }
}

async fn submit_article_update(
    data: &Data<DatabaseHandle>,
    edit_form: &EditArticleData,
    original_article: &DbArticle,
) -> Result<(), Error> {
    let edit = DbEdit::new(&original_article, &edit_form.new_text)?;
    if original_article.local {
        let updated_article = {
            let mut lock = data.articles.lock().unwrap();
            let article = lock.get_mut(edit_form.ap_id.inner()).unwrap();
            article.text = edit_form.new_text.clone();
            article.latest_version = edit.version.clone();
            article.edits.push(edit.clone());
            article.clone()
        };

        UpdateLocalArticle::send(updated_article, &data).await?;
    } else {
        UpdateRemoteArticle::send(
            edit,
            original_article.instance.dereference(&data).await?,
            &data,
        )
        .await?;
    }
    Ok(())
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GetArticleData {
    pub title: String,
}

#[debug_handler]
async fn get_article(
    Query(query): Query<GetArticleData>,
    data: Data<DatabaseHandle>,
) -> MyResult<Json<DbArticle>> {
    let articles = data.articles.lock().unwrap();
    let article = articles
        .iter()
        .find(|a| a.1.title == query.title)
        .ok_or(anyhow!("not found"))?
        .1
        .clone();
    Ok(Json(article))
}

#[derive(Deserialize, Serialize)]
pub struct ResolveObject {
    pub id: Url,
}

#[debug_handler]
async fn resolve_instance(
    Query(query): Query<ResolveObject>,
    data: Data<DatabaseHandle>,
) -> MyResult<Json<DbInstance>> {
    let instance: DbInstance = ObjectId::from(query.id).dereference(&data).await?;
    Ok(Json(instance))
}

#[debug_handler]
async fn resolve_article(
    Query(query): Query<ResolveObject>,
    data: Data<DatabaseHandle>,
) -> MyResult<Json<DbArticle>> {
    let article: DbArticle = ObjectId::from(query.id).dereference(&data).await?;
    Ok(Json(article))
}

#[debug_handler]
async fn get_local_instance(data: Data<DatabaseHandle>) -> MyResult<Json<DbInstance>> {
    Ok(Json(data.local_instance()))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FollowInstance {
    pub instance_id: ObjectId<DbInstance>,
}

#[debug_handler]
async fn follow_instance(
    data: Data<DatabaseHandle>,
    Form(query): Form<FollowInstance>,
) -> MyResult<()> {
    let instance = query.instance_id.dereference(&data).await?;
    data.local_instance().follow(&instance, &data).await?;
    Ok(())
}

#[debug_handler]
async fn edit_conflicts(data: Data<DatabaseHandle>) -> MyResult<Json<Vec<Conflict>>> {
    let lock = data.conflicts.lock().unwrap();
    let conflicts = lock.clone();
    Ok(Json(conflicts))
}
