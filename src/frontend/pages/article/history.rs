use crate::frontend::{
    article_title,
    components::article_nav::ArticleNav,
    extract_domain,
    pages::article_resource,
    render_date_time,
    user_link,
};
use leptos::*;

#[component]
pub fn ArticleHistory() -> impl IntoView {
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

                                {article
                                    .edits
                                    .into_iter()
                                    .rev()
                                    .map(|edit| {
                                        let path = format!(
                                            "/article/{}@{}/diff/{}",
                                            article.article.title,
                                            extract_domain(&article.article.ap_id),
                                            edit.edit.hash.0,
                                        );
                                        view! {
                                            <li>
                                                {render_date_time(edit.edit.created)}": "
                                                <a href=path>{edit.edit.summary}</a> " by "
                                                {user_link(&edit.creator)}
                                            </li>
                                        }
                                    })
                                    .collect::<Vec<_>>()}

                            </div>
                        }
                    })
            }}

        </Suspense>
    }
}
