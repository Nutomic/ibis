use super::send_email;
use crate::{
    error::BackendResult,
    impls::{IbisContext, notifications::Notification},
};
use ibis_markdown::{render_article_markdown, render_comment_markdown};

pub(crate) async fn send_notification_email(
    notifs: Vec<Notification>,
    context: &IbisContext,
) -> BackendResult<()> {
    // TODO: reduce number of db reads
    for n in notifs {
        let data = Notification::read_data(n.id, context)?;
        if let (Some(email), true) = (data.local_user.email, data.local_user.email_notifications) {
            let article_title = data.article.title();
            let creator_title = data.creator.title();
            let notifications_link =
                format!("https://{}/notifications", &context.conf.federation.domain);

            let (subject, html) = if let Some(comment) = data.comment {
                let comment_text = render_comment_markdown(&comment.content);
                (
                    format!("New comment on article {article_title}"),
                    format!(
                        r#"<h1>Comment</h1>
                    <br>
                    <div>{creator_title} commented on "{article_title}": {comment_text}</div>
                    <br> 
                    <a href="{notifications_link}">inbox</a>"#,
                    ),
                )
            } else if let Some(edit) = data.edit {
                let edit_diff = edit.diff;
                (
                    format!("New edit on article {article_title}"),
                    format!(
                        r#"<h1>Edit</h1><br>
                    <div>{creator_title} edited "{article_title}": 
                    <pre><code>{edit_diff}</code></pre>
                    </div>
                    <br> 
                    <a href="{notifications_link}">inbox</a>"#,
                    ),
                )
            } else if data.conflict.is_some() {
                // Edit conflict, dont send notification as it should be shown on
                // website directly after user action
                continue;
            } else {
                let article_text = render_article_markdown(&data.article.text);
                (
                    format!("New article {article_title}"),
                    format!(
                        r#"<h1>New article</h1>
                    <br>
                    <div>{creator_title} created "{article_title}": {article_text}</div>
                    <br> 
                    <a href="{notifications_link}">inbox</a>"#,
                    ),
                )
            };
            send_email(&subject, &email, html, context).await?;
        }
    }
    Ok(())
}
