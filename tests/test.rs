extern crate fediwiki;
use fediwiki::federation::objects::article::DbArticle;
use fediwiki::start;

#[tokio::test]
async fn test_get_article() {
    let hostname = "localhost:8131";
    let handle = tokio::task::spawn(async {
        start(hostname).await.unwrap();
    });

    let title = "Manu_Chao";
    let res: DbArticle = reqwest::get(format!("http://{hostname}/api/v1/article/{title}"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(title, res.title);
    assert!(res.local);
    handle.abort();
}

#[tokio::test]
async fn test_follow_instance() {
    let hostname_alpha = "localhost:8131";
    let hostname_beta = "localhost:8132";
    let handle_alpha = tokio::task::spawn(async {
        start(hostname_alpha).await.unwrap();
    });
    let handle_beta = tokio::task::spawn(async {
        start(hostname_beta).await.unwrap();
    });

    handle_alpha.abort();
    handle_beta.abort();
}