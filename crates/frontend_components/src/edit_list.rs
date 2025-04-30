use crate::{
    Pending,
    utils::formatting::{article_link, edit_path, edit_time, user_link},
};
use ibis_database::common::article::EditView;
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
                        let path = edit_path(&edit.edit, &edit.article);
                        let edit_time = edit_time(edit.edit.published);
                        let second_line = if for_article {
                            Either::Left(
                                view! {
                                    {edit_time}
                                    " by "
                                    {user_link(&edit.creator)}
                                },
                            )
                        } else {
                            Either::Right(
                                view! {
                                    {edit_time}
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
                                        <Pending pending=edit.edit.pending />
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
