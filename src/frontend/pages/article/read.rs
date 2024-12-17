use crate::frontend::{
    components::article_nav::{ActiveTab, ArticleNav},
    markdown::render_markdown,
    pages::{article::table_of_contents::Toc, article_resource},
};
use leptos::prelude::*;

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
                            <div>
                                <Toc text=article.article.text.clone() />
                                <div
                                    class="max-w-full prose prose-slate"
                                    inner_html=render_markdown(&article.article.text)
                                ></div>

                            </div>
                        }
                    })
            }}

        </Suspense>
    }
}
