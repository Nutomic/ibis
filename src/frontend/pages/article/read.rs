use crate::frontend::{
    article_title,
    components::article_nav::ArticleNav,
    markdown::render_markdown,
    pages::article_resource,
};
use leptos::*;

#[component]
pub fn ReadArticle() -> impl IntoView {
    let article = article_resource();

    view! {
        <ArticleNav article=article />
        <Suspense fallback=|| {
            view! { "Loading..." }
        }>

            {move || {
                article
                    .get()
                    .map(|article| {
                        view! {
                            <div class="prose prose-slate">
                                <h1 class="slate">{article_title(&article.article)}</h1>
                                <div inner_html=render_markdown(&article.article.text)></div>
                            </div>
                        }
                    })
            }}

        </Suspense>
    }
}
