use crate::frontend::utils::errors::FrontendResult;
use crate::{
    common::{article::DbArticleView, validation::can_edit_article},
    frontend::utils::{
        formatting::{article_path, article_title},
        resources::{is_admin, is_logged_in},
    },
};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

#[derive(Clone, Copy)]
pub enum ActiveTab2 {
    Read,
    Discussion,
    History,
    Edit,
    Actions,
}

#[component]
pub fn ArticleNav2(
    article: Resource<FrontendResult<DbArticleView>>,
    active_tab: ActiveTab2,
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
                            <div class="flex flex-row">
                                <h1 class="flex-auto my-6 font-serif text-4xl font-bold grow">
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

fn tab_classes(active_tab: ActiveTab2) -> ActiveTab2Classes {
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
        ActiveTab2::Read => classes.read = TAB_ACTIVE,
        ActiveTab2::Discussion => classes.discussion = TAB_ACTIVE,
        ActiveTab2::History => classes.history = TAB_ACTIVE,
        ActiveTab2::Edit => classes.edit = TAB_ACTIVE,
        ActiveTab2::Actions => classes.actions = TAB_ACTIVE,
    }
    classes
}

fn page_title(active_tab: ActiveTab2, article_title: &str) -> String {
    let active = match active_tab {
        ActiveTab2::Read => return article_title.to_string(),
        ActiveTab2::Discussion => "Discuss",
        ActiveTab2::History => "History",
        ActiveTab2::Edit => "Edit",
        ActiveTab2::Actions => "Actions",
    };
    format!("{active} — {article_title}")
}
