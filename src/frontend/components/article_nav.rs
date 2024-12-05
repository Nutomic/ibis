use crate::{
    common::{validation::can_edit_article, ArticleView, GetInstance},
    frontend::{
        api::CLIENT,
        app::{is_admin, is_logged_in},
        article_path,
        article_title,
        components::instance_follow_button::InstanceFollowButton,
    },
};
use leptos::prelude::*;
use leptos_router::components::A;

pub enum ActiveTab {
    Read,
    History,
    Edit,
    Actions,
}

#[component]
pub fn ArticleNav(article: Resource<ArticleView>, active_tab: ActiveTab) -> impl IntoView {
    let tab_classes = tab_classes(&active_tab);

    view! {
        <Suspense>
            {move || {
                article
                    .get()
                    .map(|article_| {
                        let title = article_title(&article_.article);
                        let instance = Resource::new(
                            move || article_.article.instance_id,
                            move |instance_id| async move {
                                let form = GetInstance {
                                    id: Some(instance_id),
                                };
                                CLIENT.get_instance(&form).await.unwrap()
                            },
                        );
                        let article_link = article_path(&article_.article);
                        let article_link_ = article_link.clone();
                        let protected = article_.article.protected;
                        view! {
                            <div role="tablist" class="tabs tabs-lifted">
                                <A href=article_link.clone() {..} class=tab_classes.read>
                                    "Read"
                                </A>
                                <A
                                    href=format!("{article_link}/history")
                                    {..}
                                    class=tab_classes.history
                                >
                                    "History"
                                </A>
                                <Show when=move || {
                                    is_logged_in()
                                        && can_edit_article(&article_.article, is_admin()).is_ok()
                                }>
                                    <A
                                        href=format!("{article_link}/edit")
                                        {..}
                                        class=tab_classes.edit
                                    >
                                        "Edit"
                                    </A>
                                </Show>
                                <Suspense>
                                    <Show when=is_logged_in>
                                        <A
                                            href=format!("{article_link_}/actions")
                                            {..}
                                            class=tab_classes.actions
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
                                </Suspense>
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
