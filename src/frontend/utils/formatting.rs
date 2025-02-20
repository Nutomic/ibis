use crate::common::{
    article::{Article, Edit},
    comment::Comment,
    instance::Instance,
    user::Person,
    utils::extract_domain,
};
use chrono::{DateTime, Local, Utc};
use leptos::prelude::*;
use std::sync::OnceLock;
use timeago::Formatter;

pub fn article_path(article: &Article) -> String {
    if article.local {
        format!("/article/{}", article.title)
    } else {
        format!(
            "/article/{}@{}",
            article.title,
            extract_domain(&article.ap_id)
        )
    }
}

pub fn article_link(article: &Article) -> impl IntoView {
    let article_path = article_path(article);
    view! {
        <a class="link" href=article_path>
            {article.title.clone()}
        </a>
    }
}

pub fn article_title(article: &Article) -> String {
    let title = article.title.replace('_', " ");
    if article.local {
        title
    } else {
        format!("{}@{}", title, extract_domain(&article.ap_id))
    }
}

pub fn user_title(person: &Person) -> String {
    let name = person
        .display_name
        .clone()
        .unwrap_or(person.username.clone());
    if person.local {
        format!("@{name}")
    } else {
        format!("@{}@{}", name, extract_domain(&person.ap_id))
    }
}

pub fn user_link(person: &Person) -> impl IntoView {
    let creator_path = if person.local {
        format!("/user/{}", person.username)
    } else {
        format!(
            "/user/{}@{}",
            person.username,
            extract_domain(&person.ap_id)
        )
    };
    view! {
        <a class="link" href=creator_path>
            {user_title(person)}
        </a>
    }
}

pub fn render_date_time(date_time: DateTime<Utc>) -> String {
    date_time
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

pub fn time_ago(time: DateTime<Utc>) -> String {
    static INSTANCE: OnceLock<Formatter> = OnceLock::new();
    let secs = Utc::now().signed_duration_since(time).num_seconds();
    let duration = std::time::Duration::from_secs(secs.try_into().unwrap_or_default());
    INSTANCE.get_or_init(Formatter::new).convert(duration)
}

pub fn instance_title_with_domain(instance: &Instance) -> String {
    let name = instance.name.clone();
    let domain = instance.domain.clone();
    if let Some(name) = name {
        format!("{name} ({domain})")
    } else {
        domain
    }
}

pub fn instance_title(instance: &Instance) -> String {
    instance.name.clone().unwrap_or(instance.domain.clone())
}

pub fn instance_updated(instance: &Instance) -> String {
    if instance.local {
        "Local".to_string()
    } else {
        format!("Updated {}", time_ago(instance.last_refreshed_at))
    }
}

pub fn comment_path(comment: &Comment, article: &Article) -> String {
    let article_path = article_path(article);
    format!("{}/discussion#comment-{}", article_path, comment.id.0)
}

pub fn edit_path(edit: &Edit, article: &Article) -> String {
    format!(
        "/article/{}@{}/diff/{}",
        article.title,
        extract_domain(&article.ap_id),
        edit.hash.0,
    )
}
