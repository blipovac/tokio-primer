use mini_redis::{client, Result};

// this part starts the tokio runtime which is able to execute async code
#[tokio::main]
async fn main() -> Result<()> {
    let mut client = client::connect("127.0.0.1:6379").await?;

    client.set("hello", "world".into()).await?;

    let result = client.get("hello").await?;

    dbg!(result);

    Ok(())
}
