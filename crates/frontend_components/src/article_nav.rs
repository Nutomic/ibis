use crate::{
    Pending,
    utils::{
        formatting::article_path,
        resources::{is_admin, is_logged_in},
    },
};
use ibis_api_client::{
    CLIENT,
    errors::{FrontendResult, FrontendResultExt},
};
use ibis_database::common::article::{ArticleView, can_edit_article};
use leptos::prelude::*;
use leptos_fluent::tr;
use leptos_meta::Title;
use leptos_router::components::A;
use phosphor_leptos::{
    BELL,
    BELL_SLASH,
    BOOK,
    CHATS_CIRCLE,
    FEDIVERSE_LOGO,
    GEAR_SIX,
    Icon,
    LIST,
    LOCK_SIMPLE,
    PENCIL,
    TRASH,
};

#[derive(Clone, Copy, Debug)]
pub enum ActiveTab {
    Read,
    Discussion,
    History,
    Edit,
    Actions,
}

#[component]
pub fn ArticleNav(
    article: Resource<FrontendResult<ArticleView>>,
    active_tab: ActiveTab,
) -> impl IntoView {
    view! {
        <Suspense>
            {move || Suspend::new(async move {
                article
                    .await
                    .map(|article_| {
                        let title = article_.article.title();
                        let article_link = article_path(&article_.article);
                        let article_link_ = article_link.clone();
                        let ap_id = article_.article.ap_id.to_string();
                        let removed = article_.article.removed;
                        let protected = article_.article.protected;
                        let pending = article_.article.pending;
                        let follow_article_action = Action::new(move |_: &()| async move {
                            CLIENT
                                .follow_article(article_.article.id, !article_.following)
                                .await
                                .error_popup(|_| article.refetch());
                        });
                        let follow_title = if article_.following {
                            tr!("notification-active")
                        } else {
                            tr!("notification-inactive")
                        };
                        view! {
                            <Title text=page_title(active_tab, &title) />
                            <div class="tabs tabs-lift md:flex">
                                <A
                                    href=article_link.clone()
                                    exact=true
                                    {..}
                                    role="tab"
                                    class="tab md:flex-auto"
                                >
                                    <Icon icon=BOOK />
                                    {tr!("read-tab")}
                                </A>
                                <A
                                    href=format!("{article_link}/discussion")
                                    {..}
                                    role="tab"
                                    class="tab md:flex-auto"
                                >
                                    <Icon icon=CHATS_CIRCLE />
                                    {tr!("discussion-tab")}
                                </A>
                                <A
                                    href=format!("{article_link}/history")
                                    {..}
                                    role="tab"
                                    class="tab md:flex-auto"
                                >
                                    <Icon icon=LIST />
                                    {tr!("history-tab")}
                                </A>
                                <Show when=move || {
                                    is_logged_in()
                                        && can_edit_article(&article_.article, is_admin()).is_ok()
                                }>
                                    <A
                                        href=format!("{article_link}/edit")
                                        {..}
                                        role="tab"
                                        class="tab md:flex-auto"
                                    >
                                        <Icon icon=PENCIL />
                                        {tr!("edit-tab")}
                                    </A>
                                </Show>
                                <Suspense>
                                    <Show when=is_admin>
                                        <A
                                            href=format!("{article_link_}/actions")
                                            {..}
                                            role="tab"
                                            class="tab md:flex-auto"
                                        >
                                            <Icon icon=GEAR_SIX />
                                            {tr!("actions-tab")}
                                        </A>
                                    </Show>
                                </Suspense>
                            </div>
                            <div class="flex flex-row place-items-center gap-2">
                                <h1 class="flex-auto my-6 font-serif text-4xl font-bold grow">
                                    {title}
                                </h1>
                                <Pending pending />
                                <a href=ap_id>
                                    <Icon icon=FEDIVERSE_LOGO size="24px" />
                                </a>
                                <Show when=move || removed>
                                    <span title=tr!("article-removed")>
                                        <Icon icon=TRASH size="24px" />
                                    </span>
                                </Show>
                                <Show when=move || protected>
                                    <span title=tr!("article-protected")>
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
                                            view! { <Icon icon=BELL_SLASH size="24px" /> }
                                        }
                                    >
                                        <Icon icon=BELL size="24px" />
                                    </Show>
                                </button>
                            </div>
                        }
                    })
            })}

        </Suspense>
    }
}

fn page_title(active_tab: ActiveTab, article_title: &str) -> String {
    let active = match active_tab {
        ActiveTab::Read => return article_title.to_string(),
        ActiveTab::Discussion => tr!("discussion-tab"),
        ActiveTab::History => tr!("history-tab"),
        ActiveTab::Edit => tr!("edit-tab"),
        ActiveTab::Actions => tr!("actions-tab"),
    };
    format!("{active} — {article_title}")
}
