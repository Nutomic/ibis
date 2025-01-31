use super::errors::FrontendResult;
use crate::{
    common::{
        instance::{Options, SiteView},
        user::LocalUserView,
    },
    frontend::api::CLIENT,
};
use leptos::prelude::*;

type SiteResource = Resource<FrontendResult<SiteView>>;

pub fn site() -> SiteResource {
    site_internal().unwrap_or_else(|| Resource::new(|| (), |_| async move { CLIENT.site().await }))
}

fn site_internal() -> Option<SiteResource> {
    use_context::<Resource<FrontendResult<SiteView>>>()
}

pub fn my_profile() -> Option<LocalUserView> {
    match site_internal() {
        Some(s) => s.map(|s| s.clone().ok().map(|s| s.my_profile))??,
        None => None,
    }
}

pub fn config() -> Options {
    match site_internal() {
        Some(s) => s.map(|s| s.clone().ok().map(|s| s.config)).flatten(),
        None => None,
    }
    .unwrap_or_default()
}

pub fn is_logged_in() -> bool {
    my_profile().is_some()
}

pub fn is_admin() -> bool {
    my_profile().map(|p| p.local_user.admin).unwrap_or(false)
}
