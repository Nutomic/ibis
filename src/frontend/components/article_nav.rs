use crate::{
    common::{validation::can_edit_article, ArticleView, GetInstance},
    frontend::{
        app::GlobalState,
        article_link,
        components::instance_follow_button::InstanceFollowButton,
    },
};
use leptos::*;
use leptos_router::*;

#[component]
pub fn ArticleNav(article: Resource<Option<String>, ArticleView>) -> impl IntoView {
    view! {
      <Suspense>
        {move || {
            article
                .get()
                .map(|article_| {
                    let instance = create_local_resource(
                        move || article_.article.instance_id,
                        move |instance_id| async move {
                            let form = GetInstance {
                                id: Some(instance_id),
                            };
                            GlobalState::api_client().get_instance(&form).await.unwrap()
                        },
                    );
                    let global_state = use_context::<RwSignal<GlobalState>>().unwrap();
                    let article_link = article_link(&article_.article);
                    let article_link_ = article_link.clone();
                    let protected = article_.article.protected;
                    view! {
                      <nav class="inner">
                        <A href=article_link.clone()>"Read"</A>
                        <A href=format!("{article_link}/history")>"History"</A>
                        <Show when=move || {
                            global_state
                                .with(|state| {
                                    let is_admin = state
                                        .my_profile
                                        .as_ref()
                                        .map(|p| p.local_user.admin)
                                        .unwrap_or(false);
                                    state.my_profile.is_some()
                                        && can_edit_article(&article_.article, is_admin).is_ok()
                                })
                        }>
                          <A href=format!("{article_link}/edit")>"Edit"</A>
                        </Show>
                        <Show when=move || global_state.with(|state| state.my_profile.is_some())>
                          <A href=format!("{article_link_}/actions")>"Actions"</A>
                          {instance
                              .get()
                              .map(|i| {
                                  view! { <InstanceFollowButton instance=i.instance.clone()/> }
                              })}

                        </Show>
                        <Show when=move || protected>
                          <span title="Article can only be edited by local admins">
                            "Protected"
                          </span>
                        </Show>
                      </nav>
                    }
                })
        }}

      </Suspense>
    }
}
