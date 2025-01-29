use crate::common::instance::SiteView;
use leptos::prelude::*;

pub fn site() -> Resource<SiteView> {
    use_context::<Resource<SiteView>>().unwrap()
}

pub fn is_logged_in() -> bool {
    let site = use_context::<Resource<SiteView>>();
    if let Some(site) = site {
        site.with_default(|site| site.my_profile.is_some())
    } else {
        false
    }
}
pub fn is_admin() -> bool {
    site().with_default(|site| {
        site.my_profile
            .as_ref()
            .map(|p| p.local_user.admin)
            .unwrap_or(false)
    })
}
pub trait DefaultResource<T> {
    fn with_default<O>(&self, f: impl FnOnce(&T) -> O) -> O;
    fn get_default(&self) -> T;
}

impl<T: Default + Send + Sync + Clone> DefaultResource<T> for Resource<T> {
    fn with_default<O>(&self, f: impl FnOnce(&T) -> O) -> O {
        self.with(|x| match x {
            Some(x) => f(x),
            None => f(&T::default()),
        })
    }
    fn get_default(&self) -> T {
        match self.get() {
            Some(x) => x.clone(),
            None => T::default(),
        }
    }
}
