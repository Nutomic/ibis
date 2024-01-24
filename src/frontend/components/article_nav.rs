use crate::common::ArticleView;
use crate::frontend::app::GlobalState;
use leptos::*;
use leptos_router::*;

#[component]
pub fn ArticleNav(article: Resource<String, ArticleView>) -> impl IntoView {
    let global_state = use_context::<RwSignal<GlobalState>>().unwrap();
    view! {
        <Suspense fallback=|| view! {  "Loading..." }>
            {move || article.get().map(|article| {
                let title = article.article.title;
                view!{
        <nav class="inner">
            <A href={format!("/article/{title}")}>"Read"</A>
            <A href={format!("/article/{title}/history")}>"History"</A>
            <Show when=move || global_state.with(|state| state.my_profile.is_some())>
                <A href={format!("/article/{title}/edit")}>"Edit"</A>
            </Show>
        </nav>
            }})}
        </Suspense>
    }
}
