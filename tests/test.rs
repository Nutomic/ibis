extern crate fediwiki;
use fediwiki::federation::objects::article::DbArticle;
use fediwiki::start;

#[tokio::test]
async fn test_get_article() {
    let hostname = "localhost:8131";
    let join = tokio::task::spawn(async {
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
    join.abort();
}
