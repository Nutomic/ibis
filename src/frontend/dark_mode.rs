use chrono::{Duration, Local};
use codee::string::FromToStringCodec;
use leptos::{Signal, SignalGet, SignalGetUntracked, SignalSet, WriteSignal};
use leptos_use::{use_cookie_with_options, use_preferred_dark, SameSite, UseCookieOptions};

#[derive(Debug, Clone)]
pub struct DarkMode {
    cookie: WriteSignal<Option<bool>>,
    pub is_dark: Signal<bool>,
    pub theme: Signal<&'static str>,
}

impl DarkMode {
    pub fn init() -> Self {
        let expires = (Local::now() + Duration::days(356)).timestamp();
        let cookie_options = UseCookieOptions::default()
            .path("/")
            .expires(expires)
            .same_site(SameSite::Strict);
        let cookie =
            use_cookie_with_options::<bool, FromToStringCodec>("dark_mode", cookie_options);

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
