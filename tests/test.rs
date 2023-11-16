extern crate fediwiki;

use fediwiki::error::MyResult;
use fediwiki::federation::objects::article::DbArticle;
use fediwiki::start;

#[tokio::test]
async fn test_get_article() -> MyResult<()> {
    let hostname = "localhost:8131";
    let handle = tokio::task::spawn(async {
        start(hostname).await.unwrap();
    });

    let title = "Manu_Chao";
    let res: DbArticle = reqwest::get(format!("http://{hostname}/api/v1/article/{title}"))
        .await?
        .json()
        .await?;
    assert_eq!(title, res.title);
    assert!(res.local);
    handle.abort();
    Ok(())
}

#[tokio::test]
async fn test_follow_instance() -> MyResult<()> {
    let hostname_alpha = "localhost:8131";
    let hostname_beta = "localhost:8132";
    let handle_alpha = tokio::task::spawn(async {
        start(hostname_alpha).await.unwrap();
    });
    let handle_beta = tokio::task::spawn(async {
        start(hostname_beta).await.unwrap();
    });

    // TODO

    handle_alpha.abort();
    handle_beta.abort();
    Ok(())
}
