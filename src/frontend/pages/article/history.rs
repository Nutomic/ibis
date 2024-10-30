use crate::frontend::{
    components::article_nav::{ActiveTab, ArticleNav},
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
        <ArticleNav article=article active_tab=ActiveTab::History />
        <Suspense fallback=|| {
            view! { "Loading..." }
        }>
            {move || {
                article
                    .get()
                    .map(|article| {
                        view! {
                            <div>
                                <ul class="list-disc">
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
                                                <li class="card card-compact bg-base-100 card-bordered m-2 rounded-s">
                                                    <div class="card-body">
                                                        <a class="link link-primary text-lg w-full" href=path>
                                                            {edit.edit.summary}
                                                        </a>
                                                        <p>
                                                            {render_date_time(edit.edit.created)}" by "
                                                            {user_link(&edit.creator)}
                                                        </p>
                                                    </div>
                                                </li>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </ul>
                            </div>
                        }
                    })
            }}

        </Suspense>
    }
}
