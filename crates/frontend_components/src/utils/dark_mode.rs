use crate::utils::use_cookie;
use leptos::prelude::*;
use leptos_use::use_preferred_dark;

#[derive(Debug, Clone)]
pub struct DarkMode {
    cookie: WriteSignal<Option<bool>>,
    pub is_dark: Signal<bool>,
    pub theme: Signal<&'static str>,
}

impl DarkMode {
    pub fn init() -> Self {
        let cookie = use_cookie("dark_mode");
        let is_dark = Signal::derive(move || {
            let default = || use_preferred_dark().get_untracked();
            cookie.0.get().unwrap_or_else(default)
        });
        let theme = Signal::derive(move || if is_dark.get() { "dim" } else { "emerald" });
        Self {
            cookie: cookie.1,
            is_dark,
            theme,
        }
    }

    pub fn toggle(&mut self) {
        let new = !self.is_dark.get_untracked();
        self.cookie.set(Some(new));
    }
}
