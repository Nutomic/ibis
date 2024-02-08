use crate::common::validation::can_edit_article;
use crate::common::ArticleView;
use crate::frontend::app::GlobalState;
use leptos::*;
use leptos_router::*;

#[component]
pub fn ArticleNav(article: Resource<Option<String>, ArticleView>) -> impl IntoView {
    let global_state = use_context::<RwSignal<GlobalState>>().unwrap();
    view! {
        <Suspense>
            {move || article.get().map(|article| {
                let title = article.article.title.clone();
                view!{
                    <nav class="inner">
                        <A href={format!("/article/{title}")}>"Read"</A>
                        <A href={format!("/article/{title}/history")}>"History"</A>
                        <Show when=move || global_state.with(|state| {
                            let is_admin = state.my_profile.as_ref().map(|p| p.local_user.admin).unwrap_or(false);
                            state.my_profile.is_some() && can_edit_article(&article.article, is_admin).is_ok()
                        })>
                            <A href={format!("/article/{title}/edit")}>"Edit"</A>
                        </Show>
                    </nav>
            }})}
        </Suspense>
    }
}
