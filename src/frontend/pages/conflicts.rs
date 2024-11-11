use crate::frontend::{app::GlobalState, article_link, article_title};
use leptos::*;

#[component]
pub fn Conflicts() -> impl IntoView {
    let conflicts = create_resource(
        move || {},
        |_| async move { GlobalState::api_client().get_conflicts().await.unwrap() },
    );

    view! {
        <h1>Your unresolved edit conflicts</h1>
        <Suspense fallback=|| view! { "Loading..." }>
            <ul>
                {move || {
                    conflicts
                        .get()
                        .map(|c| {
                            c.into_iter()
                                .map(|c| {
                                    let link = format!(
                                        "{}/edit/{}",
                                        article_link(&c.article),
                                        c.id.0,
                                    );
                                    view! {
                                        <li>
                                            <a href=link>
                                                {article_title(&c.article)} " - " {c.summary}
                                            </a>
                                        </li>
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                }}

            </ul>
        </Suspense>
    }
}
