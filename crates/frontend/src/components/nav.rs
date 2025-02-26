use crate::utils::{
    dark_mode::DarkMode,
    formatting::instance_title,
    resources::{config, is_admin, is_logged_in, my_profile, site},
};
use ibis_api_client::{CLIENT, errors::FrontendResultExt};
use leptos::{IntoView, component, prelude::*, view, *};
use leptos_router::hooks::use_navigate;
use phosphor_leptos::{
    BELL_RINGING,
    CARDS,
    EXCLAMATION_MARK,
    GEAR,
    HOUSE,
    Icon,
    MAGNIFYING_GLASS,
    PLUS,
};

#[component]
pub fn Nav() -> impl IntoView {
    let logout_action = Action::new(move |_| async move {
        CLIENT.logout().await.error_popup(|_| site().refetch());
    });
    let notification_count = Resource::new(
        || (),
        move |_| async move { CLIENT.notifications_count().await.unwrap_or_default() },
    );

    let (search_query, set_search_query) = signal(String::new());
    let mut dark_mode = expect_context::<DarkMode>();
    view! {
        <nav class="p-2.5 border-b border-solid md:fixed md:w-64 md:h-full max-sm:navbar max-sm: border-slate-400 md:border-e">
            <div
                id="navbar-start"
                class="md:h-full max-sm:navbar-start max-sm:flex max-sm:dropdown max-sm:dropdown-bottom max-sm:dropdown-end max-sm:w-full"
            >
                <a href="/" class="flex flex-row md:hidden">
                    <img src="/logo.png" class="m-auto size-8" />
                    <h2 class="m-2 font-serif text-xl font-bold">
                        {move || Suspend::new(async move {
                            site().await.map(|s| instance_title(&s.instance))
                        })}
                    </h2>
                </a>
                <div class="flex-grow md:hidden"></div>
                <button class="lg:hidden btn btn-outline">Menu</button>
                <div class="md:h-full menu dropdown-content max-sm:rounded-box max-sm:z-[1] max-sm:shadow">
                    <Transition>
                        <a href="/" class="max-sm:hidden">
                            <img src="/logo.png" class="m-auto" />
                            <h2 class="m-4 font-serif text-xl font-bold">
                                {move || Suspend::new(async move {
                                    site().await.map(|s| instance_title(&s.instance))
                                })}
                            </h2>
                        </a>
                        <ul>
                            <li>
                                <a href="/">
                                    <Icon icon=HOUSE />
                                    "Main Page"
                                </a>
                            </li>
                            <li>
                                <a href="/explore">
                                    <Icon icon=CARDS />
                                    "Explore"
                                </a>
                            </li>
                            <li>
                                <a href="/about">
                                    <Icon icon=EXCLAMATION_MARK />
                                    "About"
                                </a>
                            </li>
                            <Show when=is_logged_in>
                                <li>
                                    <a href="/create-article">
                                        <Icon icon=PLUS />
                                        "Create Article"
                                    </a>
                                </li>
                                <li>
                                    <a href="/notifications">
                                        <Icon icon=BELL_RINGING />
                                        "Notifications "
                                        <span class="indicator-item indicator-end badge badge-neutral">
                                            {notification_count}
                                        </span>
                                    </a>
                                </li>
                            </Show>
                            <Show when=is_admin>
                                <li>
                                    <a href="/settings">
                                        <Icon icon=GEAR />
                                        "Settings"
                                    </a>
                                </li>
                            </Show>
                            <li>
                                <form
                                    class="p-1 m-0 form-control"
                                    on:submit=move |ev| {
                                        ev.prevent_default();
                                        let navigate = use_navigate();
                                        let query = search_query.get();
                                        if !query.is_empty() {
                                            navigate(
                                                &format!("/search?query={query}"),
                                                Default::default(),
                                            );
                                        }
                                    }
                                >
                                    <input
                                        type="text"
                                        class="w-full rounded input input-secondary input-bordered input-xs"
                                        placeholder="Search"
                                        prop:value=search_query
                                        on:keyup=move |ev: ev::KeyboardEvent| {
                                            let val = event_target_value(&ev);
                                            set_search_query.update(|v| *v = val);
                                        }
                                    />

                                    <button class="btn btn-xs btn-secondary">
                                        <Icon icon=MAGNIFYING_GLASS size="18px" />
                                    </button>
                                </form>
                            </li>
                        </ul>
                        <div class="divider"></div>
                        <Show
                            when=is_logged_in
                            fallback=move || {
                                view! {
                                    <li>
                                        <a href="/login">"Login"</a>
                                    </li>
                                    <Show when=move || config().registration_open>
                                        <li>
                                            <a href="/register">"Register"</a>
                                        </li>
                                    </Show>
                                }
                            }
                        >

                            {my_profile()
                                .map(|my_profile| {
                                    let profile_link = format!(
                                        "/user/{}",
                                        my_profile.person.username,
                                    );
                                    view! {
                                        <p class="self-center">
                                            "Logged in as " <a class="link" href=profile_link>
                                                {my_profile.person.username}
                                            </a>
                                        </p>
                                        <a class="self-center py-2 link" href="/edit_profile">
                                            Edit Profile
                                        </a>
                                        <button
                                            class="self-center w-min btn btn-outline btn-xs"
                                            on:click=move |_| {
                                                logout_action.dispatch(());
                                            }
                                        >
                                            Logout
                                        </button>
                                    }
                                })}

                        </Show>
                        <div class="grow min-h-2"></div>
                        <div class="grid gap-2 m-1">
                            <label class="flex gap-2 cursor-pointer">
                                <span class="label-text">Light</span>
                                <input
                                    type="checkbox"
                                    class="toggle"
                                    prop:checked=dark_mode.is_dark
                                    on:click=move |_| { dark_mode.toggle() }
                                />
                                <span class="label-text">Dark</span>
                            </label>
                            <p>"Version "{env!("CARGO_PKG_VERSION")}</p>
                            <p>
                                <a href="https://github.com/Nutomic/ibis" class="link">
                                    Source Code
                                </a>
                            </p>
                        </div>
                    </Transition>
                </div>
            </div>
        </nav>
    }
}
