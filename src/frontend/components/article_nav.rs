use crate::{
    common::{validation::can_edit_article, ArticleView, GetInstance},
    frontend::{
        app::GlobalState,
        article_link,
        article_title,
        components::instance_follow_button::InstanceFollowButton,
    },
};use crate::frontend::api::CLIENT;
use leptos::*;
use leptos_router::*;

pub enum ActiveTab {
    Read,
    History,
    Edit,
    Actions,
}

#[component]
pub fn ArticleNav(
    article: Resource<Option<String>, ArticleView>,
    active_tab: ActiveTab,
) -> impl IntoView {
    let tab_classes = tab_classes(&active_tab);

    view! {
        <Suspense>
            {move || {
                article
                    .get()
                    .map(|article_| {
                        let title = article_title(&article_.article);
                        let instance = create_local_resource(
                            move || article_.article.instance_id,
                            move |instance_id| async move {
                                let form = GetInstance {
                                    id: Some(instance_id),
                                };
                                CLIENT.get_instance(&form).await.unwrap()
                            },
                        );
                        let global_state = use_context::<RwSignal<GlobalState>>().unwrap();
                        let article_link = article_link(&article_.article);
                        let article_link_ = article_link.clone();
                        let protected = article_.article.protected;
                        view! {
                            <div role="tablist" class="tabs tabs-lifted">
                                <A class=tab_classes.read href=article_link.clone()>
                                    "Read"
                                </A>
                                <A class=tab_classes.history href=format!("{article_link}/history")>
                                    "History"
                                </A>
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
                                    <A class=tab_classes.edit href=format!("{article_link}/edit")>
                                        "Edit"
                                    </A>
                                </Show>
                                <Show when=move || {
                                    global_state.with(|state| state.my_profile.is_some())
                                }>
                                    <A
                                        class=tab_classes.actions
                                        href=format!("{article_link_}/actions")
                                    >
                                        "Actions"
                                    </A>
                                    {instance
                                        .get()
                                        .map(|i| {
                                            view! {
                                                <InstanceFollowButton instance=i.instance.clone() />
                                            }
                                        })}

                                </Show>
                            </div>
                            <div class="flex flex-row">
                                <h1 class="text-4xl font-bold font-serif my-6 grow flex-auto">
                                    {title}
                                </h1>
                                <Show when=move || protected>
                                    <span
                                        class="place-self-center"
                                        title="Article can only be edited by local admins"
                                    >
                                        "Protected"
                                    </span>
                                </Show>
                            </div>
                        }
                    })
            }}

        </Suspense>
    }
}

struct ActiveTabClasses {
    read: &'static str,
    history: &'static str,
    edit: &'static str,
    actions: &'static str,
}

fn tab_classes(active_tab: &ActiveTab) -> ActiveTabClasses {
    const TAB_INACTIVE: &str = "tab";
    const TAB_ACTIVE: &str = "tab tab-active";
    let mut classes = ActiveTabClasses {
        read: TAB_INACTIVE,
        history: TAB_INACTIVE,
        edit: TAB_INACTIVE,
        actions: TAB_INACTIVE,
    };
    match active_tab {
        ActiveTab::Read => classes.read = TAB_ACTIVE,
        ActiveTab::History => classes.history = TAB_ACTIVE,
        ActiveTab::Edit => classes.edit = TAB_ACTIVE,
        ActiveTab::Actions => classes.actions = TAB_ACTIVE,
    }
    classes
}
