use crate::frontend::{
    components::article_nav::{ActiveTab, ArticleNav},
    markdown::render_markdown,
    pages::article_resource,
};
use leptos::prelude::*;

use crate::frontend::pages::article::table_of_contents;

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
                                id="table-of-contents" 
                                inner_html=table_of_contents::generate_table_of_contents(&article.article.text)
                            >
                                </div>
                            <div
                                class="max-w-full prose prose-slate"
                                inner_html=render_markdown(&article.article.text)
                            >
                                </div>
                        }
                    })
            }}

        </Suspense>
    }
}
