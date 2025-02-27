use super::{UserExt, check_is_admin};
use crate::utils::generate_article_version;
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use anyhow::anyhow;
use axum::{Form, Json, extract::Query};
use axum_macros::debug_handler;
use chrono::Utc;
use diffy::{Patch, apply, create_patch, merge};
use ibis_api_client::{
    article::{
        CreateArticleParams,
        DeleteConflictParams,
        EditArticleParams,
        FollowArticleParams,
        ForkArticleParams,
        GetArticleParams,
        GetConflictParams,
        ListArticlesParams,
        ProtectArticleParams,
        RemoveArticleParams,
    },
    instance::SearchArticleParams,
};
use ibis_database::{
    common::{
        ResolveObjectParams,
        SuccessResponse,
        article::{
            ApiConflict,
            Article,
            ArticleView,
            Conflict,
            Edit,
            EditVersion,
            can_edit_article,
        },
        instance::Instance,
        utils::{extract_domain, http_protocol_str},
    },
    error::BackendResult,
    impls::{IbisContext, article::DbArticleForm, conflict::DbConflictForm, edit::DbEditForm},
};
use ibis_federate::{
    activities::{article::create_article::CreateArticle, submit_article_update},
    objects::article::ArticleWrapper,
    validate::{validate_article_title, validate_not_empty},
};
use url::Url;

/// Create a new article with empty text, and federate it to followers.
#[debug_handler]
pub(crate) async fn create_article(
    user: UserExt,
    context: Data<IbisContext>,
    Form(mut params): Form<CreateArticleParams>,
) -> BackendResult<Json<ArticleView>> {
    params.title = validate_article_title(&params.title)?;
    validate_not_empty(&params.text)?;

    let local_instance = Instance::read_local(&context)?;
    let ap_id = Url::parse(&format!(
        "{}://{}/article/{}",
        http_protocol_str(),
        extract_domain(&local_instance.ap_id.into()),
        params.title
    ))?
    .into();
    let form = DbArticleForm {
        title: params.title,
        text: String::new(),
        ap_id,
        instance_id: local_instance.id,
        local: true,
        protected: false,
    };
    let article = Article::create(form, user.person.id, &context)?;

    let edit_data = EditArticleParams {
        article_id: article.id,
        new_text: params.text,
        summary: params.summary,
        previous_version_id: article.latest_edit_version(&context)?,
        resolve_conflict_id: None,
    };

    let _ = edit_article(
        UserExt {
            local_user_view: user.clone(),
        },
        context.reset_request_count(),
        Form(edit_data),
    )
    .await?;

    Article::follow(article.id, &user, &context)?;

    // allow reading unapproved article here
    let article_view = Article::read_view(article.id, Some(&user), &context)?;
    CreateArticle::send_to_followers(article_view.article.clone().into(), &context).await?;

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
pub(crate) async fn edit_article(
    user: UserExt,
    context: Data<IbisContext>,
    Form(mut params): Form<EditArticleParams>,
) -> BackendResult<Json<Option<ApiConflict>>> {
    validate_not_empty(&params.new_text)?;
    // resolve conflict if any
    if let Some(resolve_conflict_id) = params.resolve_conflict_id {
        Conflict::delete(resolve_conflict_id, user.person.id, &context)?;
    }
    let original_article = Article::read_view(params.article_id, Some(&user), &context)?;
    if params.new_text == original_article.article.text {
        return Err(anyhow!("Edit contains no changes").into());
    }
    if params.summary.is_empty() {
        return Err(anyhow!("No summary given").into());
    }
    can_edit_article(&original_article.article, user.local_user.admin)?;
    // ensure trailing newline for clean diffs
    if !params.new_text.ends_with('\n') {
        params.new_text.push('\n');
    }
    let local_link = format!("](https://{}", context.config.federation.domain);
    if params.new_text.contains(&local_link) {
        return Err(anyhow!("Links to local instance don't work over federation").into());
    }

    // Markdown formatting
    let new_text = fmtm::format(&params.new_text, Some(80))?;

    if params.previous_version_id == original_article.latest_version {
        // No intermediate changes, simply submit new version
        submit_article_update(
            new_text.clone(),
            params.summary.clone(),
            params.previous_version_id,
            &original_article.article,
            user.person.id,
            &context,
        )
        .await?;
        Ok(Json(None))
    } else {
        // There have been other changes since this edit was initiated. Get the common ancestor
        // version and generate a diff to find out what exactly has changed.
        let edits = Edit::list_for_article(original_article.article.id, &context)?;
        let ancestor = generate_article_version(&edits, &params.previous_version_id)?;
        let patch = create_patch(&ancestor, &new_text);

        let previous_version = Edit::read(&params.previous_version_id, &context)?;
        let form = DbConflictForm {
            hash: EditVersion::new(&patch.to_string()),
            diff: patch.to_string(),
            summary: params.summary.clone(),
            creator_id: user.person.id,
            article_id: original_article.article.id,
            previous_version_id: previous_version.hash,
        };
        let conflict = Conflict::create(&form, &context)?;
        Ok(Json(
            db_conflict_to_api_conflict(conflict, true, &context).await?,
        ))
    }
}

/// Retrieve an article by ID. It must already be stored in the local database.
#[debug_handler]
pub(crate) async fn get_article(
    user: Option<UserExt>,
    Query(query): Query<GetArticleParams>,
    context: Data<IbisContext>,
) -> BackendResult<Json<ArticleView>> {
    let user = user.map(|u| u.inner());
    match (query.title, query.id) {
        (Some(title), None) => Ok(Json(Article::read_view(
            (&title, query.domain),
            user.as_ref(),
            &context,
        )?)),
        (None, Some(id)) => {
            if query.domain.is_some() {
                return Err(anyhow!("Cant combine id and instance_domain").into());
            }
            let article = Article::read_view(id, user.as_ref(), &context)?;
            Ok(Json(article))
        }
        _ => Err(anyhow!("Must pass exactly one of title, id").into()),
    }
}

#[debug_handler]
pub(crate) async fn list_articles(
    user: UserExt,
    Query(query): Query<ListArticlesParams>,
    context: Data<IbisContext>,
) -> BackendResult<Json<Vec<Article>>> {
    let include_removed = user.local_user.admin && query.include_removed.unwrap_or_default();
    Ok(Json(Article::read_all(
        query.only_local,
        query.instance_id,
        include_removed,
        &context,
    )?))
}

/// Fork a remote article to local instance. This is useful if there are disagreements about
/// how an article should be edited.
#[debug_handler]
pub(crate) async fn fork_article(
    user: UserExt,
    context: Data<IbisContext>,
    Form(mut params): Form<ForkArticleParams>,
) -> BackendResult<Json<ArticleView>> {
    // TODO: lots of code duplicated from create_article(), can move it into helper
    let original_article = Article::read_view(params.article_id, Some(&user), &context)?;
    params.new_title = validate_article_title(&params.new_title)?;

    let local_instance = Instance::read_local(&context)?;
    let ap_id = Url::parse(&format!(
        "{}://{}/article/{}",
        http_protocol_str(),
        extract_domain(&local_instance.ap_id.into()),
        &params.new_title
    ))?
    .into();
    let form = DbArticleForm {
        title: params.new_title,
        text: original_article.article.text.clone(),
        ap_id,
        instance_id: local_instance.id,
        local: true,
        protected: false,
    };
    let article = Article::create(form, user.person.id, &context)?;

    // copy edits to new article
    // this could also be done in sql

    let edits = Edit::list_for_article(original_article.article.id, &context)?;
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
            published: Utc::now(),
            pending: false,
        };
        Edit::create(&form, &context)?;
    }

    Article::follow(article.id, &user, &context)?;

    CreateArticle::send_to_followers(article.clone().into(), &context).await?;

    Ok(Json(Article::read_view(article.id, Some(&user), &context)?))
}

/// Fetch a remote article, including edits collection. Allows viewing and editing. Note that new
/// article changes can only be received if we follow the instance, or if it is refetched manually.
#[debug_handler]
pub(super) async fn resolve_article(
    user: UserExt,
    Query(query): Query<ResolveObjectParams>,
    context: Data<IbisContext>,
) -> BackendResult<Json<ArticleView>> {
    let article: ArticleWrapper = ObjectId::from(query.id).dereference(&context).await?;
    Ok(Json(Article::read_view(article.id, Some(&user), &context)?))
}

/// Search articles for matching title or body text.
#[debug_handler]
pub(super) async fn search_article(
    Query(query): Query<SearchArticleParams>,
    context: Data<IbisContext>,
) -> BackendResult<Json<Vec<Article>>> {
    if query.query.is_empty() {
        return Err(anyhow!("Query is empty").into());
    }
    let article = Article::search(&query.query, &context)?;
    Ok(Json(article))
}

#[debug_handler]
pub(crate) async fn protect_article(
    user: UserExt,
    context: Data<IbisContext>,
    Form(params): Form<ProtectArticleParams>,
) -> BackendResult<Json<Article>> {
    check_is_admin(&user)?;
    let article = Article::update_protected(params.article_id, params.protected, &context)?;
    Ok(Json(article))
}

#[debug_handler]
pub async fn remove_article(
    user: UserExt,
    context: Data<IbisContext>,
    Form(params): Form<RemoveArticleParams>,
) -> BackendResult<Json<()>> {
    check_is_admin(&user)?;
    Article::update_removed(params.article_id, params.remove, &context)?;
    Ok(Json(()))
}

#[debug_handler]
pub async fn get_conflict(
    user: UserExt,
    context: Data<IbisContext>,
    Form(params): Form<GetConflictParams>,
) -> BackendResult<Json<ApiConflict>> {
    let conflict = Conflict::read(params.conflict_id, user.person.id, &context)?;
    let conflict = db_conflict_to_api_conflict(conflict, true, &context)
        .await?
        .ok_or(anyhow!("Patch was applied cleanly"))?;
    Ok(Json(conflict))
}

#[debug_handler]
pub async fn delete_conflict(
    user: UserExt,
    context: Data<IbisContext>,
    Form(params): Form<DeleteConflictParams>,
) -> BackendResult<Json<()>> {
    Conflict::delete(params.conflict_id, user.person.id, &context)?;
    Ok(Json(()))
}

#[debug_handler]
pub(crate) async fn follow_article(
    user: UserExt,
    context: Data<IbisContext>,
    Form(params): Form<FollowArticleParams>,
) -> BackendResult<Json<SuccessResponse>> {
    if params.follow {
        Article::follow(params.id, &user, &context)?;
    } else {
        Article::unfollow(params.id, &user, &context)?;
    }
    Ok(Json(SuccessResponse::default()))
}

pub async fn db_conflict_to_api_conflict(
    conflict: Conflict,
    force_dereference: bool,
    context: &Data<IbisContext>,
) -> BackendResult<Option<ApiConflict>> {
    let article = Article::read_view(conflict.article_id, None, context)?;
    let ap_id = ObjectId::<ArticleWrapper>::from(article.article.ap_id);
    let original_article = if force_dereference {
        // Make sure to get latest version from origin so that all conflicts can be resolved
        ap_id.dereference_forced(context).await?
    } else {
        ap_id.dereference(context).await?
    };

    // create common ancestor version
    let edits = Edit::list_for_article(original_article.id, context)?;
    let ancestor = generate_article_version(&edits, &conflict.previous_version_id)?;

    let patch = Patch::from_str(&conflict.diff)?;
    // apply self.diff to ancestor to get `ours`
    let ours = apply(&ancestor, &patch)?;
    match merge(&ancestor, &ours, &original_article.text) {
        Ok(new_text) => {
            // patch applies cleanly so we are done, federate the change
            submit_article_update(
                new_text,
                conflict.summary.clone(),
                conflict.previous_version_id.clone(),
                &original_article,
                conflict.creator_id,
                context,
            )
            .await?;
            Conflict::delete(conflict.id, conflict.creator_id, context)?;
            Ok(None)
        }
        Err(three_way_merge) => {
            // there is a merge conflict, user needs to do three-way-merge
            Ok(Some(ApiConflict {
                id: conflict.id,
                hash: conflict.hash.clone(),
                three_way_merge,
                summary: conflict.summary.clone(),
                article: original_article.clone().0,
                previous_version_id: original_article.latest_edit_version(context)?,
                published: conflict.published,
            }))
        }
    }
}
