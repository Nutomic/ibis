use crate::frontend::app::GlobalState;
use leptos::{component, use_context, view, IntoView, RwSignal, SignalWith, *};
use leptos_router::*;

#[component]
pub fn Nav() -> impl IntoView {
    let global_state = use_context::<RwSignal<GlobalState>>().unwrap();
    let logout_action = create_action(move |_| async move {
        GlobalState::api_client().logout().await.unwrap();
        GlobalState::update_my_profile();
    });
    let registration_open = create_local_resource(
        || (),
        move |_| async move {
            GlobalState::api_client()
                .get_local_instance()
                .await
                .map(|i| i.registration_open)
                .unwrap_or_default()
        },
    );

    let (search_query, set_search_query) = create_signal(String::new());
    view! {
      <nav class="inner" style="min-width: 250px;">
        <li>
          <A href="/">"Main Page"</A>
        </li>
        <li>
          <A href="/article/list">"List Articles"</A>
        </li>
        <Show when=move || global_state.with(|state| state.my_profile.is_some())>
          <li>
            <A href="/article/create">"Create Article"</A>
          </li>
          <li>
            <A href="/conflicts">"Edit Conflicts"</A>
          </li>
        </Show>
        <li>
          <form on:submit=move |ev| {
              ev.prevent_default();
              let navigate = leptos_router::use_navigate();
              let query = search_query.get();
              if !query.is_empty() {
                  navigate(&format!("/search?query={query}"), Default::default());
              }
          }>
            <input
              type="text"
              placeholder="Search"
              prop:value=search_query
              on:keyup=move |ev: ev::KeyboardEvent| {
                  let val = event_target_value(&ev);
                  set_search_query.update(|v| *v = val);
              }
            />

            <button>Go</button>
          </form>
        </li>
        <Show
          when=move || global_state.with(|state| state.my_profile.is_some())
          fallback=move || {
              view! {
                <li>
                  <A href="/login">"Login"</A>
                </li>
                <Show when=move || registration_open.get().unwrap_or_default()>
                  <li>
                    <A href="/register">"Register"</A>
                  </li>
                </Show>
              }
          }
        >

          {
              let my_profile = global_state.with(|state| state.my_profile.clone().unwrap());
              let profile_link = format!("/user/{}", my_profile.person.username);
              view! {
                <p>
                  "Logged in as "
                  <a
                    href=profile_link
                    style="border: none; padding: 0; color: var(--accent) !important;"
                  >
                    {my_profile.person.username}
                  </a>
                </p>
                <button on:click=move |_| logout_action.dispatch(())>Logout</button>
              }
          }

        </Show>
      </nav>
    }
}
