extern crate redis;
use redis::AsyncCommands;

async fn do_something() -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_async_connection().await?;

    /* do something here */
    let res: Option<String> = con.get("bar").await.expect("get fail");
    println!("{:?}", res);

    let res: Result<Option<String>, redis::RedisError> = con.get("foo").await;
    println!("{:?}", res);

    Ok(())
}

#[tokio::main]
async fn main() {
    _ = do_something().await;
    println!("Hello, world!");
}
