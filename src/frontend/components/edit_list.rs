use crate::{
    common::{article::EditView, utils::extract_domain},
    frontend::utils::formatting::{article_link, render_date_time, user_link},
};
use leptos::{either::Either, prelude::*};

// If `for_article` is true, edit entries link to the respective user account. Otherwise
// if edits for a user is rendered, entries link to the respective article.
#[component]
pub fn EditList(edits: Vec<EditView>, for_article: bool) -> impl IntoView {
    view! {
        <div>
            <ul class="list-disc">
                {edits
                    .into_iter()
                    .rev()
                    .map(|edit: EditView| {
                        let path = format!(
                            "/article/{}@{}/diff/{}",
                            edit.article.title,
                            extract_domain(&edit.article.ap_id),
                            edit.edit.hash.0,
                        );
                        let date = render_date_time(edit.edit.published);
                        let second_line = if for_article {
                            Either::Left(
                                view! {
                                    {date}
                                    " by "
                                    {user_link(&edit.creator)}
                                },
                            )
                        } else {
                            Either::Right(
                                view! {
                                    {date}
                                    " on "
                                    {article_link(&edit.article)}
                                },
                            )
                        };
                        view! {
                            <li class="m-2 card card-compact bg-base-100 card-bordered rounded-s">
                                <div class="card-body">
                                    <div class="flex w-full">
                                        <a class="text-lg grow link link-primary" href=path>
                                            {edit.edit.summary}
                                        </a>
                                        <Show when=move || edit.edit.pending>
                                            <span class="p-1 w-min rounded border-2 border-rose-300">
                                                Pending
                                            </span>
                                        </Show>
                                    </div>
                                    <p>{second_line}</p>
                                </div>
                            </li>
                        }
                    })
                    .collect::<Vec<_>>()}
            </ul>
        </div>
    }
}
