use crate::{
    common::{
        article::{ApiConflict, DbArticleView, EditArticleParams},
        newtypes::ConflictId,
        MAIN_PAGE_NAME,
    },
    frontend::{
        api::CLIENT,
        components::{
            article_editor::EditorView,
            article_nav::{ActiveTab, ArticleNav},
            suspense_error::SuspenseError,
        },
        pages::article_resource,
    },
};
use chrono::{Days, Utc};
use leptos::{html::Textarea, prelude::*};
use leptos_router::{
    components::Redirect,
    hooks::{use_params_map, use_query_map},
};
use leptos_use::{use_textarea_autosize, UseTextareaAutosizeReturn};

#[derive(Clone, PartialEq)]
enum EditResponse {
    None,
    Success,
    Conflict(ApiConflict),
}

const CONFLICT_MESSAGE: &str = "There was an edit conflict. Resolve it manually and resubmit.";

#[component]
pub fn EditArticle() -> impl IntoView {
    let article = article_resource();

    let (edit_response, set_edit_response) = signal(EditResponse::None);
    let (edit_error, set_edit_error) = signal(None::<String>);

    let conflict = Resource::new(
        || use_query_map().get_untracked().get("conflict_id").clone(),
        move |conflict_id| async move {
            if let Some(conflict_id) = conflict_id {
                let conflict_id = conflict_id.parse().map(ConflictId)?;
                let conflict = CLIENT.get_conflict(conflict_id).await?;
                set_edit_response.set(EditResponse::Conflict(conflict));
                set_edit_error.set(Some(CONFLICT_MESSAGE.to_string()));
            }
            Ok(())
        },
    );

    let textarea_ref = NodeRef::<Textarea>::new();
    let UseTextareaAutosizeReturn {
        content,
        set_content,
        trigger_resize: _,
    } = use_textarea_autosize(textarea_ref);
    let (summary, set_summary) = signal(String::new());
    let (wait_for_response, set_wait_for_response) = signal(false);
    let button_is_disabled =
        Signal::derive(move || wait_for_response.get() || summary.get().is_empty());
    let submit_action = Action::new(
        move |(new_text, summary, article, edit_response): &(
            String,
            String,
            DbArticleView,
            EditResponse,
        )| {
            let new_text = new_text.clone();
            let summary = summary.clone();
            let article = article.clone();
            let resolve_conflict_id = match edit_response {
                EditResponse::Conflict(conflict) => Some(conflict.id),
                _ => None,
            };
            let previous_version_id = match edit_response {
                EditResponse::Conflict(conflict) => conflict.previous_version_id.clone(),
                _ => article.latest_version,
            };
            async move {
                set_edit_error.update(|e| *e = None);
                let params = EditArticleParams {
                    article_id: article.article.id,
                    new_text,
                    summary,
                    previous_version_id,
                    resolve_conflict_id,
                };
                set_wait_for_response.update(|w| *w = true);
                let res = CLIENT.edit_article(&params).await;
                set_wait_for_response.update(|w| *w = false);
                match res {
                    Ok(Some(conflict)) => {
                        set_edit_response.update(|v| *v = EditResponse::Conflict(conflict));
                        set_edit_error.set(Some(CONFLICT_MESSAGE.to_string()));
                    }
                    Ok(None) => {
                        set_edit_response.update(|v| *v = EditResponse::Success);
                    }
                    Err(err) => {
                        let msg = err.to_string();
                        log::warn!("Unable to edit: {msg}");
                        set_edit_error.update(|e| *e = Some(msg));
                    }
                }
            }
        },
    );

    view! {
        <ArticleNav article=article active_tab=ActiveTab::Edit />
        <Show
            when=move || edit_response.get() == EditResponse::Success
            fallback=move || {
                view! {
                    <SuspenseError result=article>
                        {move || Suspend::new(async move {
                            article
                                .await
                                .map(|mut article| {
                                    if let EditResponse::Conflict(conflict) = edit_response.get() {
                                        article.article.text = conflict.three_way_merge;
                                        set_summary.set(conflict.summary);
                                    }
                                    set_content.set(article.article.text.clone());
                                    let article_ = article.clone();
                                    let show_federation_warning = !article.instance.local
                                        && article.instance.last_refreshed_at + Days::new(3)
                                            < Utc::now();
                                    view! {
                                        // set initial text, otherwise submit with no changes results in empty text
                                        <div>
                                            {move || {
                                                edit_error
                                                    .get()
                                                    .map(|err| {
                                                        view! { <p class="alert alert-error">{err}</p> }
                                                    })
                                            }} <Show when=move || show_federation_warning>
                                                <div class="alert alert-warning">
                                                    "This article is hosted on "
                                                    {article.instance.domain.clone()}
                                                    " which hasnt been federated in "
                                                    {(Utc::now() - article.instance.last_refreshed_at)
                                                        .num_days()}
                                                    " days. Edits will most likely fail. Instead consider forking the article to your local instance (under Actions), or edit a different article."
                                                </div>
                                            </Show> <EditorView textarea_ref content set_content />
                                            <div class="flex flex-row mr-2">
                                                <input
                                                    type="text"
                                                    class="input input-primary grow me-4"
                                                    placeholder="Edit summary"
                                                    value=summary.get_untracked()
                                                    on:keyup=move |ev| {
                                                        let val = event_target_value(&ev);
                                                        set_summary.update(|p| *p = val);
                                                    }
                                                />

                                                <button
                                                    class="btn btn-primary"
                                                    prop:disabled=move || button_is_disabled.get()
                                                    on:click=move |_| {
                                                        submit_action
                                                            .dispatch((
                                                                content.get(),
                                                                summary.get(),
                                                                article_.clone(),
                                                                edit_response.get(),
                                                            ));
                                                    }
                                                >

                                                    Submit
                                                </button>
                                            </div>
                                        </div>
                                    }
                                })
                        })}
                    </SuspenseError>
                }
            }
        >
            <Redirect path={
                let params = use_params_map();
                let title = params.get().get("title").clone().unwrap_or(MAIN_PAGE_NAME.to_string());
                format!("/article/{title}?edit_successful")
            } />
        </Show>
        <SuspenseError result=conflict>""</SuspenseError>
    }
}
