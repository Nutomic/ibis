use reqwest::Client;use once_cell::sync::Lazy;use anyhow::anyhow;

pub static CLIENT: Lazy<Client> = Lazy::new(Client::new);

pub async fn create_article(instance: &IbisInstance, title: String) -> MyResult<ArticleView> {
    let create_form = CreateArticleData {
        title: title.clone(),
    };
    let req = CLIENT
        .post(format!("http://{}/api/v1/article", &instance.hostname))
        .form(&create_form)
        .bearer_auth(&instance.jwt);
    let article: ArticleView = handle_json_res(req).await?;

    // create initial edit to ensure that conflicts are generated (there are no conflicts on empty file)
    let edit_form = EditArticleData {
        article_id: article.article.id,
        new_text: TEST_ARTICLE_DEFAULT_TEXT.to_string(),
        previous_version_id: article.latest_version,
        resolve_conflict_id: None,
    };
    edit_article(instance, &edit_form).await
}

pub async fn get_article(hostname: &str, article_id: i32) -> MyResult<ArticleView> {
    let get_article = GetArticleData { article_id };
    get_query::<ArticleView, _>(hostname, "article", Some(get_article.clone())).await
}

pub async fn edit_article_with_conflict(
    instance: &IbisInstance,
    edit_form: &EditArticleData,
) -> MyResult<Option<ApiConflict>> {
    let req = CLIENT
        .patch(format!("http://{}/api/v1/article", instance.hostname))
        .form(edit_form)
        .bearer_auth(&instance.jwt);
    handle_json_res(req).await
}

pub async fn get_conflicts(instance: &IbisInstance) -> MyResult<Vec<ApiConflict>> {
    let req = CLIENT
        .get(format!(
            "http://{}/api/v1/edit_conflicts",
            &instance.hostname
        ))
        .bearer_auth(&instance.jwt);
    handle_json_res(req).await
}

pub async fn edit_article(
    instance: &IbisInstance,
    edit_form: &EditArticleData,
) -> MyResult<ArticleView> {
    let edit_res = edit_article_with_conflict(instance, edit_form).await?;
    assert!(edit_res.is_none());
    get_article(&instance.hostname, edit_form.article_id).await
}

pub async fn get<T>(hostname: &str, endpoint: &str) -> MyResult<T>
    where
        T: for<'de> Deserialize<'de>,
{
    get_query(hostname, endpoint, None::<i32>).await
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
    handle_json_res(req).await
}

pub async fn fork_article(
    instance: &IbisInstance,
    form: &ForkArticleData,
) -> MyResult<ArticleView> {
    let req = CLIENT
        .post(format!("http://{}/api/v1/article/fork", instance.hostname))
        .form(form)
        .bearer_auth(&instance.jwt);
    handle_json_res(req).await
}

pub async fn handle_json_res<T>(req: RequestBuilder) -> MyResult<T>
    where
        T: for<'de> Deserialize<'de>,
{
    let res = req.send().await?;
    let status = res.status();
    let text = res.text().await?;
    if status == StatusCode::OK {
        Ok(serde_json::from_str(&text).map_err(|e| anyhow!("Json error on {text}: {e}"))?)
    } else {
        Err(anyhow!("API error: {text}").into())
    }
}

pub async fn follow_instance(instance: &IbisInstance, follow_instance: &str) -> MyResult<()> {
    // fetch beta instance on alpha
    let resolve_form = ResolveObject {
        id: Url::parse(&format!("http://{}", follow_instance))?,
    };
    let instance_resolved: DbInstance =
        get_query(&instance.hostname, "resolve_instance", Some(resolve_form)).await?;

    // send follow
    let follow_form = FollowInstance {
        id: instance_resolved.id,
    };
    // cant use post helper because follow doesnt return json
    let res = CLIENT
        .post(format!(
            "http://{}/api/v1/instance/follow",
            instance.hostname
        ))
        .form(&follow_form)
        .bearer_auth(&instance.jwt)
        .send()
        .await?;
    if res.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(anyhow!("API error: {}", res.text().await?).into())
    }
}

pub async fn register(hostname: &str, username: &str, password: &str) -> MyResult<LoginResponse> {
    let register_form = RegisterUserData {
        username: username.to_string(),
        password: password.to_string(),
    };
    let req = CLIENT
        .post(format!("http://{}/api/v1/user/register", hostname))
        .form(&register_form);
    handle_json_res(req).await
}

pub async fn login(
    instance: &IbisInstance,
    username: &str,
    password: &str,
) -> MyResult<LoginResponse> {
    let login_form = LoginUserData {
        username: username.to_string(),
        password: password.to_string(),
    };
    let req = CLIENT
        .post(format!("http://{}/api/v1/user/login", instance.hostname))
        .form(&login_form);
    handle_json_res(req).await
}
