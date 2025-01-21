use crate::{
    common::{
        article::{ForkArticleParams, ProtectArticleParams},
        newtypes::ArticleId,
    },
    frontend::{
        api::CLIENT,
        app::is_admin,
        article_path,
        components::article_nav::{ActiveTab, ArticleNav},
        pages::article_resource,
        DbArticle,
    },
};
use leptos::{ev::KeyboardEvent, prelude::*};
use leptos_router::components::Redirect;

#[component]
pub fn ArticleActions() -> impl IntoView {
    let article = article_resource();
    let (new_title, set_new_title) = signal(String::new());
    let (fork_response, set_fork_response) = signal(Option::<DbArticle>::None);
    let (error, set_error) = signal(None::<String>);
    let fork_action = Action::new(move |(article_id, new_title): &(ArticleId, String)| {
        let params = ForkArticleParams {
            article_id: *article_id,
            new_title: new_title.to_string(),
        };
        async move {
            set_error.update(|e| *e = None);
            let result = CLIENT.fork_article(&params).await;
            match result {
                Ok(res) => set_fork_response.set(Some(res.article)),
                Err(err) => {
                    set_error.update(|e| *e = Some(err.to_string()));
                }
            }
        }
    });
    let protect_action = Action::new(move |(id, protected): &(ArticleId, bool)| {
        let params = ProtectArticleParams {
            article_id: *id,
            protected: !protected,
        };
        async move {
            set_error.update(|e| *e = None);
            let result = CLIENT.protect_article(&params).await;
            match result {
                Ok(_res) => article.refetch(),
                Err(err) => {
                    set_error.update(|e| *e = Some(err.to_string()));
                }
            }
        }
    });
    view! {
        <ArticleNav article=article active_tab=ActiveTab::Actions />
        <Suspense fallback=|| {
            view! { "Loading..." }
        }>
            {move || {
                article
                    .get()
                    .map(|article| {
                        view! {
                            <div>
                                {move || {
                                    error
                                        .get()
                                        .map(|err| {
                                            view! { <p class="alert">{err}</p> }
                                        })
                                }} <Show when=move || { is_admin() && article.article.local }>
                                    <button
                                        class="btn btn-secondary"
                                        on:click=move |_| {
                                            protect_action
                                                .dispatch((article.article.id, article.article.protected));
                                        }
                                    >
                                        Toggle Article Protection
                                    </button>
                                    <p>"Protect a local article so that only admins can edit it"</p>
                                </Show> <Show when=move || !article.article.local>
                                    <input
                                        class="input"
                                        placeholder="New Title"
                                        on:keyup=move |ev: KeyboardEvent| {
                                            let val = event_target_value(&ev);
                                            set_new_title.update(|v| *v = val);
                                        }
                                    />

                                    <button
                                        class="btn"
                                        disabled=move || new_title.get().is_empty()
                                        on:click=move |_| {
                                            fork_action.dispatch((article.article.id, new_title.get()));
                                        }
                                    >

                                        Fork Article
                                    </button>
                                    <p>
                                        "You can fork a remote article to the local instance. This is useful if the original
                                        instance is dead, or if there are disagreements how the article should be written."
                                    </p>
                                </Show>
                            </div>
                        }
                    })
            }}

        </Suspense>
        <Show when=move || fork_response.get().is_some()>
            <Redirect path=article_path(&fork_response.get().unwrap()) />
        </Show>
        <p>"TODO: add option for admin to delete article etc"</p>
    }
}
