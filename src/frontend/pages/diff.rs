use crate::frontend::{
    article_title,
    components::article_nav::{ActiveTab, ArticleNav},
    pages::{article_edits_resource, article_resource},
    render_date_time,
    user_link,
};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

#[component]
pub fn EditDiff() -> impl IntoView {
    let params = use_params_map();
    let article = article_resource();
    let edits = article_edits_resource(article);

    view! {
        <ArticleNav article=article active_tab=ActiveTab::History />
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
                let pending = edit.edit.pending;
                let title = format!(
                    "Diff {} — {}",
                    edit.edit.summary,
                    article_title(&article.await.article),
                );
                view! {
                    <Title text=title />
                    <div class="flex w-full">
                        <h2 class="my-2 font-serif text-xl font-bold grow">{label}</h2>
                        <Show when=move || pending>
                            <span class="p-1 w-min rounded border-2 border-rose-300 h-min">
                                Pending
                            </span>
                        </Show>
                    </div>
                    <p>"by " {user_link(&edit.creator)}</p>
                    <div class="p-2 my-2 bg-gray-200 rounded">
                        <pre class="text-wrap">
                            <code>{edit.edit.diff.clone()}</code>
                        </pre>
                    </div>
                }
            })}

        </Suspense>
    }
}
