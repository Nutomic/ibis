use crate::common::GetArticleData;
use crate::common::LocalUserView;
use crate::common::{ArticleView, LoginUserData, RegisterUserData};
use crate::frontend::error::MyResult;
use anyhow::anyhow;
use once_cell::sync::Lazy;
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};

pub static CLIENT: Lazy<Client> = Lazy::new(Client::new);

pub async fn get_article(hostname: &str, title: String) -> MyResult<ArticleView> {
    let get_article = GetArticleData { title };
    get_query::<ArticleView, _>(hostname, "article", Some(get_article.clone())).await
}

pub async fn get_query<T, R>(hostname: &str, endpoint: &str, query: Option<R>) -> MyResult<T>
where
    T: for<'de> Deserialize<'de>,
    R: Serialize,
{
    let mut req = CLIENT.get(format!("http://{}/api/v1/{}", hostname, endpoint));
    if let Some(query) = query {
        req = req.query(&query);
    }
    handle_json_res::<T>(req).await
}

pub async fn handle_json_res<T>(req: RequestBuilder) -> MyResult<T>
where
    T: for<'de> Deserialize<'de>,
{
    let res = req.send().await?;
    let status = res.status();
    let text = res.text().await?;
    if status == reqwest::StatusCode::OK {
        Ok(serde_json::from_str(&text).map_err(|e| anyhow!("Json error on {text}: {e}"))?)
    } else {
        Err(anyhow!("API error: {text}").into())
    }
}

pub async fn register(hostname: &str, register_form: RegisterUserData) -> MyResult<LocalUserView> {
    let req = CLIENT
        .post(format!("http://{}/api/v1/account/register", hostname))
        .form(&register_form);
    handle_json_res::<LocalUserView>(req).await
}

pub async fn login(hostname: &str, login_form: LoginUserData) -> MyResult<LocalUserView> {
    let req = CLIENT
        .post(format!("http://{}/api/v1/account/login", hostname))
        .form(&login_form);
    handle_json_res::<LocalUserView>(req).await
}

pub async fn my_profile(hostname: &str) -> MyResult<LocalUserView> {
    let req = CLIENT.get(format!("http://{}/api/v1/account/my_profile", hostname));
    handle_json_res::<LocalUserView>(req).await
}

pub async fn logout(hostname: &str) -> MyResult<()> {
    CLIENT
        .get(format!("http://{}/api/v1/account/logout", hostname))
        .send()
        .await?;
    Ok(())
}
