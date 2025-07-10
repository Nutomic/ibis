use ibis_api_client::{CLIENT, errors::FrontendError};
use ibis_database::common::newtypes::CommentId;
use ibis_frontend_components::suspense_error::SuspenseError;
use leptos::prelude::*;
use leptos_router::{components::Redirect, hooks::use_params_map};

#[component]
pub fn CommentRedirect() -> impl IntoView {
    let comment_view = Resource::new(
        move || {
            let params = use_params_map();
            params.get().get("id").clone()
        },
        move |id| async move {
            let id = CommentId(
                id.ok_or(FrontendError::new("missing comment id"))?
                    .parse()?,
            );
            CLIENT.get_comment(id).await
        },
    );

    view! {
        <SuspenseError result=comment_view>
            {move || Suspend::new(async move {
                comment_view
                    .await
                    .map(|comment_view| {
                        let title = comment_view.article.title().replace(' ', "_");
                        let path = format!("/article/{title}/discussion");
                        view! { <Redirect path=path /> }
                    })
            })}
        </SuspenseError>
    }
}
