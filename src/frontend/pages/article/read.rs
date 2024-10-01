use crate::frontend::{
    article_title,
    components::article_nav::ArticleNav,
    markdown::markdown_parser,
    pages::article_resource,
};
use leptos::*;

#[component]
pub fn ReadArticle() -> impl IntoView {
    let article = article_resource();

    view! {
      <ArticleNav article=article/>
      <Suspense fallback=|| {
          view! { "Loading..." }
      }>

        {
            let parser = markdown_parser();
            move || {
                article
                    .get()
                    .map(|article| {
                        view! {
                          <div class="item-view">
                            <h1>{article_title(&article.article)}</h1>
                            <div inner_html=parser.parse(&article.article.text).render()></div>
                          </div>
                        }
                    })
            }
        }

      </Suspense>
    }
}
