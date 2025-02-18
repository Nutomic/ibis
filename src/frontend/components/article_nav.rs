use crate::{
    common::{article::DbArticleView, validation::can_edit_article},
    frontend::{
        api::CLIENT,
        utils::{
            errors::{FrontendResult, FrontendResultExt},
            formatting::{article_path, article_title},
            resources::{is_admin, is_logged_in},
        },
    },
};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;
use phosphor_leptos::{Icon, BELL, BELL_SLASH, LOCK_SIMPLE};

#[derive(Clone, Copy)]
pub enum ActiveTab {
    Read,
    Discussion,
    History,
    Edit,
    Actions,
}

#[component]
pub fn ArticleNav(
    article: Resource<FrontendResult<DbArticleView>>,
    active_tab: ActiveTab,
) -> impl IntoView {
    let tab_classes = tab_classes(active_tab);

    view! {
        <Suspense>
            {move || Suspend::new(async move {
                article
                    .await
                    .map(|article_| {
                        let title = article_title(&article_.article);
                        let article_link = article_path(&article_.article);
                        let article_link_ = article_link.clone();
                        let protected = article_.article.protected;
                        let follow_article_action = Action::new(move |_: &()| async move {
                            CLIENT
                                .follow_article(article_.article.id, !article_.following)
                                .await
                                .error_popup(|_| article.refetch());
                        });
                        let follow_title = if article_.following {
                            "Stop notifications"
                        } else {
                            "Get notified about new article edits and comments"
                        };
                        view! {
                            <Title text=page_title(active_tab, &title) />
                            <div role="tablist" class="tabs tabs-lifted">
                                <A href=article_link.clone() {..} class=tab_classes.read>
                                    "Read"
                                </A>
                                <A
                                    href=format!("{article_link}/discussion")
                                    {..}
                                    class=tab_classes.discussion
                                >
                                    "Discussion"
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
                                    </Show>
                                </Suspense>
                            </div>
                            <div class="flex flex-row place-items-center">
                                <h1 class="flex-auto my-6 font-serif text-4xl font-bold grow">
                                    {title}
                                </h1>
                                <Show when=move || protected>
                                    <span
                                        class="mr-2"
                                        title="Article can only be edited by local admins"
                                    >
                                        <Icon icon=LOCK_SIMPLE size="24px" />
                                    </span>
                                </Show>
                                <button
                                    class="btn btn-sm btn-outline"
                                    on:click=move |_| {
                                        follow_article_action.dispatch(());
                                    }
                                    title=follow_title
                                >
                                    <Show
                                        when=move || article_.following
                                        fallback=move || {
                                            view! { <Icon icon=BELL size="24px" /> }
                                        }
                                    >
                                        <Icon icon=BELL_SLASH size="24px" />
                                    </Show>
                                </button>
                            </div>
                        }
                    })
            })}

        </Suspense>
    }
}

struct ActiveTab2Classes {
    read: &'static str,
    discussion: &'static str,
    history: &'static str,
    edit: &'static str,
    actions: &'static str,
}

fn tab_classes(active_tab: ActiveTab) -> ActiveTab2Classes {
    const TAB_INACTIVE: &str = "tab";
    const TAB_ACTIVE: &str = "tab tab-active";
    let mut classes = ActiveTab2Classes {
        read: TAB_INACTIVE,
        discussion: TAB_INACTIVE,
        history: TAB_INACTIVE,
        edit: TAB_INACTIVE,
        actions: TAB_INACTIVE,
    };
    match active_tab {
        ActiveTab::Read => classes.read = TAB_ACTIVE,
        ActiveTab::Discussion => classes.discussion = TAB_ACTIVE,
        ActiveTab::History => classes.history = TAB_ACTIVE,
        ActiveTab::Edit => classes.edit = TAB_ACTIVE,
        ActiveTab::Actions => classes.actions = TAB_ACTIVE,
    }
    classes
}

fn page_title(active_tab: ActiveTab, article_title: &str) -> String {
    let active = match active_tab {
        ActiveTab::Read => return article_title.to_string(),
        ActiveTab::Discussion => "Discuss",
        ActiveTab::History => "History",
        ActiveTab::Edit => "Edit",
        ActiveTab::Actions => "Actions",
    };
    format!("{active} — {article_title}")
}
