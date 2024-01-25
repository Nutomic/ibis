use crate::frontend::components::article_nav::ArticleNav;
use crate::frontend::pages::article_resource;
use leptos::*;
use leptos_router::*;
use markdown_it::MarkdownIt;

#[component]
pub fn ReadArticle() -> impl IntoView {
    let params = use_params_map();
    let title = params
        .get_untracked()
        .get("title")
        .cloned()
        .unwrap_or("Main_Page".to_string());
    let article = article_resource(title);

    view! {
        <ArticleNav article=article.clone()/>
        <Suspense fallback=|| view! {  "Loading..." }> {
            let parser = markdown_parser();
            move || article.get().map(|article|
            view! {
                <div class="item-view">
                    <h1>{article.article.title.replace('_', " ")}</h1>
                    <div inner_html={parser.parse(&article.article.text).render()}/>
                </div>
            })
        }
        </Suspense>
    }
}

fn markdown_parser() -> MarkdownIt {
    let mut parser = MarkdownIt::new();
    markdown_it::plugins::cmark::add(&mut parser);
    markdown_it::plugins::extra::add(&mut parser);
    parser
}
