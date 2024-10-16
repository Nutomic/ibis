use crate::frontend::{
    components::article_nav::{ActiveTab, ArticleNav},
    markdown::render_markdown,
    pages::article_resource,
};
use leptos::*;

#[component]
pub fn ReadArticle() -> impl IntoView {
    let article = article_resource();

    view! {
        <ArticleNav article=article active_tab=ActiveTab::Read />
        <Suspense fallback=|| {
            view! { "Loading..." }
        }>

            {move || {
                article
                    .get()
                    .map(|article| {
                        view! {
                            <div
                                class="prose prose-slate"
                                inner_html=render_markdown(&article.article.text)
                            ></div>
                        }
                    })
            }}

        </Suspense>
    }
}
