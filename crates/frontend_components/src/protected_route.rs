use crate::utils::resources::is_logged_in;
use leptos::prelude::*;
use leptos_router::{
    MatchNestedRoutes,
    PossibleRouteMatch,
    SsrMode,
    any_nested_route::IntoAnyNestedRoute,
    components::{ProtectedRoute, ProtectedRouteProps},
};

#[component(transparent)]
pub fn IbisProtectedRoute<Segments, ViewFn, View>(
    path: Segments,
    view: ViewFn,
    #[prop(optional)] ssr: SsrMode,
) -> impl MatchNestedRoutes + Clone
where
    ViewFn: Fn() -> View + Send + Clone + 'static,
    View: IntoView + 'static,
    Segments: PossibleRouteMatch + Clone + Send + 'static,
{
    let condition = move || Some(is_logged_in());
    let redirect_path = || "/";
    let props = ProtectedRouteProps {
        path,
        view,
        condition,
        redirect_path,
        ssr,
        fallback: Default::default(),
    };
    ProtectedRoute(props).into_any_nested_route()
}
