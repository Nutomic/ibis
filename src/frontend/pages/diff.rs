use crate::frontend::{
    components::article_nav::{ActiveTab, ArticleNav},
    pages::article_resource,
    render_date_time,
    user_link,
};
use leptos::*;
use leptos_router::*;

#[component]
pub fn EditDiff() -> impl IntoView {
    let params = use_params_map();
    let article = article_resource();

    view! {
        <ArticleNav article=article active_tab=ActiveTab::Edit />
        <Suspense fallback=|| {
            view! { "Loading..." }
        }>
            {move || {
                article
                    .get()
                    .map(|article| {
                        let hash = params.get_untracked().get("hash").cloned().unwrap();
                        let edit = article
                            .edits
                            .iter()
                            .find(|e| e.edit.hash.0.to_string() == hash)
                            .unwrap();
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
                    })
            }}

        </Suspense>
    }
}
