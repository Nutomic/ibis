use super::{check_is_admin, UserExt};
use crate::{
    backend::{
        database::{
            article::DbArticleForm,
            conflict::{DbConflict, DbConflictForm},
            edit::DbEditForm,
            IbisContext,
        },
        federation::activities::{create_article::CreateArticle, submit_article_update},
        utils::{
            error::BackendResult,
            generate_article_version,
            validate::{validate_article_title, validate_not_empty},
        },
    },
    common::{
        article::{
            ApiConflict,
            ApproveArticleParams,
            Article,
            ArticleView,
            CreateArticleParams,
            DeleteConflictParams,
            Edit,
            EditArticleParams,
            EditVersion,
            FollowArticleParams,
            ForkArticleParams,
            GetArticleParams,
            GetConflictParams,
            ListArticlesParams,
            ProtectArticleParams,
            SearchArticleParams,
        },
        instance::Instance,
        utils::{extract_domain, http_protocol_str},
        validation::can_edit_article,
        ResolveObjectParams,
        SuccessResponse,
    },
};
use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use anyhow::anyhow;
use axum::{extract::Query, Form, Json};
use axum_macros::debug_handler;
use chrono::Utc;
use diffy::create_patch;

/// Create a new article with empty text, and federate it to followers.
#[debug_handler]
pub(in crate::backend::api) async fn create_article(
    user: UserExt,
    context: Data<IbisContext>,
    Form(mut params): Form<CreateArticleParams>,
) -> BackendResult<Json<ArticleView>> {
    params.title = validate_article_title(&params.title)?;
    validate_not_empty(&params.text)?;

    let local_instance = Instance::read_local(&context)?;
    let ap_id = ObjectId::parse(&format!(
        "{}://{}/article/{}",
        http_protocol_str(),
        extract_domain(&local_instance.ap_id),
        params.title
    ))?;
    let form = DbArticleForm {
        title: params.title,
        text: String::new(),
        ap_id,
        instance_id: local_instance.id,
        local: true,
        protected: false,
        approved: !context.config.options.article_approval,
    };
    let article = Article::create(form, &context)?;

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
    CreateArticle::send_to_followers(article_view.article.clone(), &context).await?;

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
    user: UserExt,
    context: Data<IbisContext>,
    Form(mut params): Form<EditArticleParams>,
) -> BackendResult<Json<Option<ApiConflict>>> {
    validate_not_empty(&params.new_text)?;
    // resolve conflict if any
    if let Some(resolve_conflict_id) = params.resolve_conflict_id {
        DbConflict::delete(resolve_conflict_id, user.person.id, &context)?;
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
        let conflict = DbConflict::create(&form, &context)?;
        Ok(Json(conflict.to_api_conflict(true, &context).await?))
    }
}

/// Retrieve an article by ID. It must already be stored in the local database.
#[debug_handler]
pub(in crate::backend::api) async fn get_article(
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
pub(in crate::backend::api) async fn list_articles(
    Query(query): Query<ListArticlesParams>,
    context: Data<IbisContext>,
) -> BackendResult<Json<Vec<Article>>> {
    Ok(Json(Article::read_all(
        query.only_local,
        query.instance_id,
        &context,
    )?))
}

/// Fork a remote article to local instance. This is useful if there are disagreements about
/// how an article should be edited.
#[debug_handler]
pub(in crate::backend::api) async fn fork_article(
    user: UserExt,
    context: Data<IbisContext>,
    Form(mut params): Form<ForkArticleParams>,
) -> BackendResult<Json<ArticleView>> {
    // TODO: lots of code duplicated from create_article(), can move it into helper
    let original_article = Article::read_view(params.article_id, Some(&user), &context)?;
    params.new_title = validate_article_title(&params.new_title)?;

    let local_instance = Instance::read_local(&context)?;
    let ap_id = ObjectId::parse(&format!(
        "{}://{}/article/{}",
        http_protocol_str(),
        extract_domain(&local_instance.ap_id),
        &params.new_title
    ))?;
    let form = DbArticleForm {
        title: params.new_title,
        text: original_article.article.text.clone(),
        ap_id,
        instance_id: local_instance.id,
        local: true,
        protected: false,
        approved: !context.config.options.article_approval,
    };
    let article = Article::create(form, &context)?;

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

    CreateArticle::send_to_followers(article.clone(), &context).await?;

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
    let article: Article = ObjectId::from(query.id).dereference(&context).await?;
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
pub(in crate::backend::api) async fn protect_article(
    user: UserExt,
    context: Data<IbisContext>,
    Form(params): Form<ProtectArticleParams>,
) -> BackendResult<Json<Article>> {
    check_is_admin(&user)?;
    let article = Article::update_protected(params.article_id, params.protected, &context)?;
    Ok(Json(article))
}

#[debug_handler]
pub async fn approve_article(
    user: UserExt,
    context: Data<IbisContext>,
    Form(params): Form<ApproveArticleParams>,
) -> BackendResult<Json<()>> {
    check_is_admin(&user)?;
    if params.approve {
        Article::update_approved(params.article_id, true, &context)?;
    } else {
        Article::delete(params.article_id, &context)?;
    }
    Ok(Json(()))
}

#[debug_handler]
pub async fn get_conflict(
    user: UserExt,
    context: Data<IbisContext>,
    Form(params): Form<GetConflictParams>,
) -> BackendResult<Json<ApiConflict>> {
    let conflict = DbConflict::read(params.conflict_id, user.person.id, &context)?;
    let conflict = conflict
        .to_api_conflict(true, &context)
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
    DbConflict::delete(params.conflict_id, user.person.id, &context)?;
    Ok(Json(()))
}

#[debug_handler]
pub(in crate::backend::api) async fn follow_article(
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
