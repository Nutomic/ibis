use crate::frontend::{
    components::{
        article_nav::{ActiveTab, ArticleNav},
        suspense_error::SuspenseError,
    },
    pages::{article_edits_resource, article_resource},
    utils::formatting::{article_title, render_date_time, user_link},
};
use leptos::{either::Either, prelude::*};
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

#[component]
pub fn EditDiff() -> impl IntoView {
    let params = use_params_map();
    let article = article_resource();
    let edits = article_edits_resource(article);

    view! {
        <ArticleNav article=article active_tab=ActiveTab::History />
        <SuspenseError result=article>
            {move || Suspend::new(async move {
                let article_title = article
                    .await
                    .map(|a| article_title(&a.article))
                    .unwrap_or_default();
                edits
                    .await
                    .map(|edits| {
                        let hash = params.get_untracked().get("hash").clone();
                        let edit = edits.iter().find(|e| Some(e.edit.hash.0.to_string()) == hash);
                        if let Some(edit) = edit {
                            let label = format!(
                                "{} ({})",
                                edit.edit.summary,
                                render_date_time(edit.edit.published),
                            );
                            let pending = edit.edit.pending;
                            let title = format!("Diff {} — {}", edit.edit.summary, article_title);
                            Either::Left(
                                view! {
                                    <Title text=title />
                                    <div class="flex w-full">
                                        <h2 class="my-2 font-serif text-xl font-bold grow">
                                            {label}
                                        </h2>
                                        <Show when=move || pending>
                                            <span class="p-1 w-min rounded border-2 border-rose-300 h-min">
                                                Pending
                                            </span>
                                        </Show>
                                    </div>
                                    <p>"by " {user_link(&edit.creator)}</p>
                                    <div class="max-w-full prose prose-slate">
                                        <pre class="text-wrap">
                                            <code>{edit.edit.diff.clone()}</code>
                                        </pre>
                                    </div>
                                },
                            )
                        } else {
                            Either::Right(
                                view! {
                                    <div class="grid place-items-center h-screen">
                                        <div class="alert alert-error w-fit">Invalid edit</div>
                                    </div>
                                },
                            )
                        }
                    })
            })}

        </SuspenseError>
    }
}
