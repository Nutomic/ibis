use crate::{
    common::{ApiConflict, ArticleView, EditArticleForm},
    frontend::{
        app::GlobalState,
        article_title,
        components::article_nav::ArticleNav,
        pages::article_resource,
    },
};
use leptos::*;
use leptos_router::use_params_map;

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
    let (edit_response, set_edit_response) = create_signal(EditResponse::None);
    let (edit_error, set_edit_error) = create_signal(None::<String>);

    let conflict_id = move || use_params_map().get().get("conflict_id").cloned();
    if let Some(conflict_id) = conflict_id() {
        create_action(move |conflict_id: &String| {
            let conflict_id: i32 = conflict_id.parse().unwrap();
            async move {
                let conflict = GlobalState::api_client()
                    .get_conflicts()
                    .await
                    .unwrap()
                    .into_iter()
                    .find(|c| c.id == conflict_id)
                    .unwrap();
                set_edit_response.set(EditResponse::Conflict(conflict));
                set_edit_error.set(Some(CONFLICT_MESSAGE.to_string()));
            }
        })
        .dispatch(conflict_id);
    }

    let (text, set_text) = create_signal(String::new());
    let (summary, set_summary) = create_signal(String::new());
    let (wait_for_response, set_wait_for_response) = create_signal(false);
    let button_is_disabled =
        Signal::derive(move || wait_for_response.get() || summary.get().is_empty());
    let submit_action = create_action(
        move |(new_text, summary, article, edit_response): &(
            String,
            String,
            ArticleView,
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
                let form = EditArticleForm {
                    article_id: article.article.id,
                    new_text,
                    summary,
                    previous_version_id,
                    resolve_conflict_id,
                };
                set_wait_for_response.update(|w| *w = true);
                let res = GlobalState::api_client()
                    .edit_article_with_conflict(&form)
                    .await;
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
                        let msg = err.0.to_string();
                        log::warn!("Unable to edit: {msg}");
                        set_edit_error.update(|e| *e = Some(msg));
                    }
                }
            }
        },
    );

    view! {
        <ArticleNav article=article />
        <Show
            when=move || edit_response.get() == EditResponse::Success
            fallback=move || {
                view! {
                    <Suspense fallback=|| {
                        view! { "Loading..." }
                    }>
                        {move || {
                            article
                                .get()
                                .map(|mut article| {
                                    if let EditResponse::Conflict(conflict) = edit_response.get() {
                                        article.article.text = conflict.three_way_merge;
                                        set_summary.set(conflict.summary);
                                    }
                                    set_text.set(article.article.text.clone());
                                    let article_ = article.clone();
                                    let rows = article.article.text.lines().count() + 1;
                                    view! {
                                        // set initial text, otherwise submit with no changes results in empty text
                                        <div id="edit-article" class="item-view">
                                            <h1>{article_title(&article.article)}</h1>
                                            {move || {
                                                edit_error
                                                    .get()
                                                    .map(|err| {
                                                        view! { <p style="color:red;">{err}</p> }
                                                    })
                                            }}

                                            <textarea id="edit-article-textarea" rows=rows on:keyup=move |ev| {
                                                let val = event_target_value(&ev);
                                                set_text.update(|p| *p = val);
                                            }>{article.article.text.clone()}</textarea>
                                            <div>
                                                <a href="https://commonmark.org/help/" target="blank_">
                                                    Markdown
                                                </a>
                                                " formatting is supported"
                                            </div>
                                            <div class="inputs">
                                            <input
                                                type="text"
                                                placeholder="Edit summary"
                                                value=summary.get_untracked()
                                                on:keyup=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    set_summary.update(|p| *p = val);
                                                }
                                            />

                                            <button
                                                prop:disabled=move || button_is_disabled.get()
                                                on:click=move |_| {
                                                    submit_action
                                                        .dispatch((
                                                            text.get(),
                                                            summary.get(),
                                                            article_.clone(),
                                                            edit_response.get(),
                                                        ))
                                                }
                                            >

                                                Submit
                                            </button>
                                            </div>
                                        </div>
                                    }
                                })
                        }}

                    </Suspense>
                }
            }
        >

            Edit successful!
        </Show>
    }
}
