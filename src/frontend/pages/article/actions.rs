use crate::{
    common::ForkArticleForm,
    frontend::{
        app::GlobalState,
        article_link,
        article_title,
        components::article_nav::ArticleNav,
        pages::article_resource,
        DbArticle,
    },
};
use leptos::*;
use leptos_router::Redirect;

#[component]
pub fn ArticleActions() -> impl IntoView {
    let article = article_resource();
    let (new_title, set_new_title) = create_signal(String::new());
    let (fork_response, set_fork_response) = create_signal(Option::<DbArticle>::None);
    let (error, set_error) = create_signal(None::<String>);
    let fork_action = create_action(move |(article_id, new_title): &(i32, String)| {
        let params = ForkArticleForm {
            article_id: *article_id,
            new_title: new_title.to_string(),
        };
        async move {
            set_error.update(|e| *e = None);
            let result = GlobalState::api_client().fork_article(&params).await;
            match result {
                Ok(res) => set_fork_response.set(Some(res.article)),
                Err(err) => {
                    set_error.update(|e| *e = Some(err.0.to_string()));
                }
            }
        }
    });
    // TODO: show fork article option (with option to set different title). after forking do redirect

    view! {
        <ArticleNav article=article/>
        <Suspense fallback=|| view! {  "Loading..." }> {
            move || article.get().map(|article|
            view! {
                <div class="item-view">
                    <h1>{article_title(&article.article)}</h1>
                    {move || {
                        error
                            .get()
                            .map(|err| {
                                view! { <p style="color:red;">{err}</p> }
                            })
                    }}
                    <Show when=move || !article.article.local>
                        <input
                            placeholder="New Title"
                            on:keyup=move |ev: ev::KeyboardEvent| {
                                let val = event_target_value(&ev);
                                set_new_title.update(|v| *v = val);
                        } />
                        <button
                            disabled=move || new_title.get().is_empty()
                            on:click=move |_| fork_action.dispatch((article.article.id, new_title.get()))>Fork Article</button>
                        <p>
                            "You can fork a remote article to the local instance. This is useful if the original
                            instance is dead, or if there are disagreements how the article should be written."
                        </p>
                    </Show>
                </div>
            })
        }
        </Suspense>
        <Show when=move || fork_response.get().is_some()>
            <Redirect path={article_link(&fork_response.get().unwrap())}/>
        </Show>
        <p>"TODO: add option for admin to delete article etc"</p>
    }
}
