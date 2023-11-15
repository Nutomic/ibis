use fediwiki::error::MyResult;
use fediwiki::start;

#[tokio::main]
pub async fn main() -> MyResult<()> {
    start("localhost:8131").await?;
    Ok(())
}
