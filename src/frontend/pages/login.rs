use crate::frontend::api::login;
use leptos::ev::SubmitEvent;
use leptos::*;
use log::info;

// TODO: this seems to be working, but need to implement registration also
// TODO: use leptos_form if possible
//       https://github.com/leptos-form/leptos_form/issues/18
fn do_login(ev: SubmitEvent, username: String, password: String) {
    ev.prevent_default();
    spawn_local(async move {
        let res = login("localhost:8080", &username, &password).await;
        info!("{}", res.unwrap().jwt);
    });
}

#[component]
pub fn Login() -> impl IntoView {
    let name = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());

    view! {
      <form on:submit=move |ev| do_login(ev, name.get(), password.get())>
      <div>
        <label for="username">Username: </label>
        <input
          id="username"
          type="text"
          on:input=move |ev| name.set(event_target_value(&ev))
          label="Username"
        />
      </div>

      <div>
        <label for="password">Password: </label>
        <input
          id="password"
          type="password"
          on:input=move |ev| password.set(event_target_value(&ev))
        />
      </div>

        <div>
          <button type="submit">
            "Login"
          </button>

        </div>
      </form>
    }
}
