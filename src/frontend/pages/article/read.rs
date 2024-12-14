use crate::frontend::{
    article_title,
    components::article_nav::ArticleNav,
    markdown::render_markdown,
    pages::article_resource,
};
use leptos::*;

use crate::frontend::pages::article::table_of_contents;

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
                            <div class="item-view">
                                <h1>{article_title(&article.article)}</h1>
                                <div id="table-of-contents" inner_html=table_of_contents::generate_table_of_contents(&article.article.text)></div>
                                <div inner_html=render_markdown(&article.article.text)></div>
                            </div>
                        }
                    })
            }}

        </Suspense>
    }
}
