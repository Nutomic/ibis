use leptos::{
    ev::beforeunload,
    prelude::{Get, Signal},
};
use leptos_use::{use_event_listener, use_window};

pub mod article_editor;
pub mod article_nav;
pub mod comment;
pub mod comment_editor;
pub mod edit_list;
pub mod instance_follow_button;
pub mod nav;
pub mod oauth_login_button;
pub mod protected_route;
pub mod suspense_error;
pub mod utils;

fn prevent_navigation(signal: Signal<String>) {
    // Prevent user from accidentally closing the page while editing. Doesnt prevent navigation
    // within Ibis.
    // https://github.com/Nutomic/ibis/issues/87
    let _ = use_event_listener(use_window(), beforeunload, move |evt| {
        if !signal.get().is_empty() {
            evt.stop_propagation();
            evt.prevent_default();
        }
    });
}
