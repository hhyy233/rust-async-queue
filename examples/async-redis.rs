use env_logger::Env;
use rust_async_queue::app::*;
use rust_async_queue::broker::*;
use tokio::time::sleep;
use tokio::time::Duration;

fn get_task() -> task::Task {
    task::Task::new_with_id(
        "k1".to_owned(),
        "test_task".to_owned(),
        "test payload".into(),
    )
}

async fn async_queue_client() -> Result<(), String> {
    let broker_url = "redis://127.0.0.1/";
    let rbb = RedisBrokerBuilder::new(broker_url.to_owned());
    let broker = rbb.build(10).await?;
    let queue = "asyncq:test_queue".to_owned();
    let name = "async-queue".to_owned();
    let aq = AsyncQueue::new(name, queue, broker);
    let client = aq.client()?;

    let t = get_task();
    let result = client.submit(&t).await?;
    let op = result.poll(Duration::from_secs(10)).await;
    if let Err(e) = op {
        println!("did not receive value within 10s, {}", e);
    } else {
        println!("{:?}", op.unwrap());
    }
    println!("done");
    sleep(Duration::from_secs(2)).await;
    Ok(())
}

async fn async_queue_server() -> Result<(), String> {
    let broker_url = "redis://127.0.0.1/";
    let rbb = RedisBrokerBuilder::new(broker_url.to_owned());
    let broker = rbb.build(10).await?;
    let queue = "asyncq:test_queue".to_owned();
    let name = "async-queue".to_owned();
    let aq = AsyncQueue::new(name, queue, broker);
    let server = aq.server()?;

    server.start(2).await?;
    println!("done");
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let h = tokio::spawn(async_queue_server());
    async_queue_client().await.unwrap();
    let _ = h.await.unwrap();

    println!("out");
}
