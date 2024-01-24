use crate::common::EditArticleData;
use crate::frontend::app::GlobalState;
use crate::frontend::components::article_nav::ArticleNav;
use crate::frontend::pages::article_resource;
use leptos::*;
use leptos_router::use_params_map;

#[component]
pub fn EditArticle() -> impl IntoView {
    let params = use_params_map();
    let title = params.get_untracked().get("title").cloned();
    let article = article_resource(title.unwrap());

    let (text, set_text) = create_signal(String::new());
    // TODO: set initial text, otherwise submit with no changes results in empty text
    //article.with(|article| set_text.update(article.as_ref().unwrap().article.text.clone()));
    let (summary, set_summary) = create_signal(String::new());
    let (edit_response, set_edit_response) = create_signal(None::<()>);
    let (edit_error, set_edit_error) = create_signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = create_signal(false);
    let button_is_disabled =
        Signal::derive(move || wait_for_response.get() || summary.get().is_empty());
    let submit_action = create_action(move |(new_text, summary): &(String, String)| {
        let new_text = new_text.clone();
        let summary = summary.clone();
        async move {
            let form = EditArticleData {
                article_id: article.get().unwrap().article.id,
                new_text,
                summary,
                previous_version_id: article.get().unwrap().latest_version,
                resolve_conflict_id: None,
            };
            set_wait_for_response.update(|w| *w = true);
            let res = GlobalState::api_client().edit_article(&form).await;
            set_wait_for_response.update(|w| *w = false);
            match res {
                Ok(_res) => {
                    set_edit_response.update(|v| *v = Some(()));
                    set_edit_error.update(|e| *e = None);
                }
                Err(err) => {
                    let msg = err.0.to_string();
                    log::warn!("Unable to edit: {msg}");
                    set_edit_error.update(|e| *e = Some(msg));
                }
            }
        }
    });

    view! {
        <ArticleNav article=article.clone()/>
        <Show
            when=move || edit_response.get().is_some()
            fallback=move || {
                view! {
                    <Suspense fallback=|| view! {  "Loading..." }> {
                        move || article.get().map(|article|
                            view! {
                                <div class="item-view">
                                    <h1>{article.article.title.replace('_', " ")}</h1>
                                    <textarea on:keyup=move |ev| {
                                        let val = event_target_value(&ev);
                                        set_text.update(|p| *p = val);
                                    }>
                                        {article.article.text}
                                    </textarea>
                                </div>
                                {move || {
                                    edit_error
                                        .get()
                                        .map(|err| {
                                            view! { <p style="color:red;">{err}</p> }
                                        })
                                }}
                                <input type="text" on:keyup=move |ev| {
                                    let val = event_target_value(&ev);
                                    set_summary.update(|p| *p = val);
                                }/>
                                <button
                                    prop:disabled=move || button_is_disabled.get()
                                    on:click=move |_| submit_action.dispatch((text.get(), summary.get()))>
                                    Submit
                                </button>
                            }
                        )
                    }
                    </Suspense>
                }}>
            Edit successful!
        </Show>
    }
}
