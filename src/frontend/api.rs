use crate::common::GetArticleData;
use crate::common::LocalUserView;
use crate::common::{ArticleView, LoginUserData, RegisterUserData};
use crate::frontend::error::MyResult;
use anyhow::anyhow;
use once_cell::sync::Lazy;
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};

pub static CLIENT: Lazy<Client> = Lazy::new(Client::new);

#[derive(Clone)]
pub struct ApiClient {
    // TODO: make these private
    pub client: Client,
    pub hostname: String,
}

impl ApiClient {
    pub fn new(client: Client, hostname: String) -> Self {
        Self { client, hostname }
    }

    async fn get_query<T, R>(&self, endpoint: &str, query: Option<R>) -> MyResult<T>
    where
        T: for<'de> Deserialize<'de>,
        R: Serialize,
    {
        let mut req = self
            .client
            .get(format!("http://{}/api/v1/{}", &self.hostname, endpoint));
        if let Some(query) = query {
            req = req.query(&query);
        }
        handle_json_res::<T>(req).await
    }

    pub async fn get_article(&self, data: GetArticleData) -> MyResult<ArticleView> {
        self.get_query::<ArticleView, _>("article", Some(data))
            .await
    }

    pub async fn register(&self, register_form: RegisterUserData) -> MyResult<LocalUserView> {
        let req = self
            .client
            .post(format!("http://{}/api/v1/account/register", self.hostname))
            .form(&register_form);
        handle_json_res::<LocalUserView>(req).await
    }

    pub async fn login(&self, login_form: LoginUserData) -> MyResult<LocalUserView> {
        let req = self
            .client
            .post(format!("http://{}/api/v1/account/login", self.hostname))
            .form(&login_form);
        handle_json_res::<LocalUserView>(req).await
    }
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
