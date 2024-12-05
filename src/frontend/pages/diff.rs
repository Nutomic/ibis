use crate::frontend::{
    components::article_nav::{ActiveTab, ArticleNav},
    pages::{article_edits_resource, article_resource},
    render_date_time,
    user_link,
};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn EditDiff() -> impl IntoView {
    let params = use_params_map();
    let article = article_resource();
    let edits = article_edits_resource(article);

    view! {
        <ArticleNav article=article active_tab=ActiveTab::Edit />
        <Suspense fallback=|| {
            view! { "Loading..." }
        }>
            {move || Suspend::new(async move {
                let edits = edits.await;
                let hash = params.get_untracked().get("hash").clone().unwrap();
                let edit = edits.iter().find(|e| e.edit.hash.0.to_string() == hash).unwrap();
                let label = format!(
                    "{} ({})",
                    edit.edit.summary,
                    render_date_time(edit.edit.published),
                );
                view! {
                    <h2 class="text-xl font-bold font-serif my-2">{label}</h2>
                    <p>"by " {user_link(&edit.creator)}</p>
                    <div class="bg-gray-200 p-2 rounded my-2">
                        <pre class="text-wrap">
                            <code>{edit.edit.diff.clone()}</code>
                        </pre>
                    </div>
                }
            })}

        </Suspense>
    }
}
