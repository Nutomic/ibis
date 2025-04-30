use chrono::{DateTime, Local, Utc};
use ibis_database::common::{
    article::{Article, Edit},
    comment::Comment,
    instance::{Instance, InstanceView},
    user::Person,
    utils::extract_domain,
};
use leptos::prelude::*;
use std::sync::OnceLock;
use timeago::Formatter;

pub fn article_path(article: &Article) -> String {
    let title = article.title.replace(" ", "_");
    if article.local {
        format!("/article/{}", title)
    } else {
        format!(
            "/article/{}@{}",
            title,
            extract_domain(article.ap_id.inner())
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

pub fn user_link(person: &Person) -> impl IntoView {
    let creator_path = if person.local {
        format!("/user/{}", person.username)
    } else {
        format!(
            "/user/{}@{}",
            person.username,
            extract_domain(person.ap_id.inner())
        )
    };
    view! {
        <a class="link" href=creator_path>
            {person.title()}
        </a>
    }
}

pub fn edit_time(date_time: DateTime<Utc>) -> impl IntoView {
    let absolute_time = date_time
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S %Z")
        .to_string();
    let time_ago = time_ago(date_time);
    view! { <span title=absolute_time>{time_ago}</span> }
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

pub fn instance_updated(instance: &InstanceView) -> String {
    if instance.instance.local {
        "Local".to_string()
    } else {
        // Get time of most recent edit, or fallback to last federation time
        let edited = instance
            .articles
            .iter()
            .map(|a| a.updated)
            .max()
            .unwrap_or(instance.instance.last_refreshed_at);
        format!("Edited {}", time_ago(edited))
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
        extract_domain(article.ap_id.inner()),
        edit.hash.0,
    )
}
