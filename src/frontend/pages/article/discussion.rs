use crate::{
    common::{comment::DbCommentView, newtypes::CommentId},
    frontend::{
        components::{
            article_nav::{ActiveTab, ArticleNav},
            comment::CommentView,
            comment_editor::CommentEditorView,
            suspense_error::SuspenseError,
        },
        pages::article_resource,
    },
};
use leptos::prelude::*;
use std::collections::HashMap;

#[component]
pub fn ArticleDiscussion() -> impl IntoView {
    let article = article_resource();

    let show_editor = signal(CommentId(-1));

    view! {
        <ArticleNav article=article active_tab=ActiveTab::Discussion />
        <SuspenseError result=article>
            {move || Suspend::new(async move {
                let article2 = article.await;
                view! {
                    <CommentEditorView article=article />
                    <div>
                        <For
                            each=move || {
                                article2
                                    .clone()
                                    .map(|a| build_comments_tree(a.comments))
                                    .unwrap_or_default()
                            }
                            key=|comment| comment.comment.id
                            children=move |comment: DbCommentView| {
                                view! { <CommentView article comment show_editor /> }
                            }
                        />
                    </div>
                }
            })}
        </SuspenseError>
    }
}

#[derive(Clone)]
struct CommentNode {
    view: DbCommentView,
    children: Vec<CommentNode>,
}

impl CommentNode {
    fn new(view: DbCommentView) -> Self {
        Self {
            view,
            children: vec![],
        }
    }
    /// Visit the tree depth-first to build flat array from tree.
    fn flatten(self) -> Vec<DbCommentView> {
        let mut res = vec![self.view];
        for c in self.children {
            res.append(&mut c.flatten());
        }
        res
    }
}

fn build_comments_tree(comments: Vec<DbCommentView>) -> Vec<DbCommentView> {
    // First create a map of CommentId -> CommentView
    let mut map: HashMap<CommentId, CommentNode> = comments
        .iter()
        .map(|v| (v.comment.id, CommentNode::new(v.clone())))
        .collect();
    debug_assert!(comments.len() == map.len());

    // Move top-level comments directly into tree vec. For comments having parent_id, move them
    // `children` of respective parent. This preserves existing order.
    let mut tree = Vec::<CommentNode>::new();
    for view in &comments {
        let child = map
            .get(&view.comment.id)
            .expect("get comment by id")
            .clone();
        if let Some(parent_id) = &view.comment.parent_id {
            let parent = map.get_mut(parent_id).expect("get parent comment by id");
            parent.children.push(child);
        } else {
            tree.push(child);
        }
    }
    debug_assert!(comments.len() == map.len());

    // Now convert it back to flat array with correct order for rendering
    tree.into_iter().flat_map(|t| t.flatten()).collect()
}
