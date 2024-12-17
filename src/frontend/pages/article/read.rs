use crate::frontend::{
    components::article_nav::{ActiveTab, ArticleNav},
    markdown::render_markdown,
    pages::{article::table_of_contents, article_resource},
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
                        let toc = table_of_contents::generate_table_of_contents(
                            &article.article.text,
                        );

                        view! {
                            <div>
                                {if !toc.is_empty() {
                                    view! {
                                        <div
                                            class="float-right mr-20 w-80 menu h-fit rounded-box"
                                            inner_html=toc
                                        ></div>
                                    }
                                        .into_any()
                                } else {
                                    view! {}
                                    ().into_any()
                                }}
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
