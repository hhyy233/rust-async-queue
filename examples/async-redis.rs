use rust_async_queue::app::*;
use serde::Deserialize;
use serde::Serialize;
use tokio::time::sleep;
use tokio::time::Duration;
use tracing::error;
use tracing::info;

// this is the target function we are using
// #[allow(unused)]
// fn add(x: i32, y: i32) -> i32 {
//     x + y
// }

#[derive(Clone, Serialize, Deserialize)]
struct AddParam {
    x: i32,
    y: i32,
}

struct Add {
    params: AddParam,
}

impl task::AQTask for Add {
    const NAME: &'static str = "add";
    type Params = AddParam;
    type Returns = i32;
    fn run(&self) -> task::TaskReturn<Self::Returns> {
        let x = self.params.x;
        let y = self.params.y;
        Add::_run(x, y)
    }
    fn from_params(params: Self::Params) -> Self {
        Self { params }
    }
}
impl Add {
    fn new(x: i32, y: i32) -> signature::Signature<Self> {
        signature::Signature::<Add>::new(AddParam { x, y })
    }
    fn _run(x: i32, y: i32) -> task::TaskReturn<i32> {
        Ok(x + y)
    }
}

async fn async_queue_client(client: Client) -> Result<(), String> {
    let t = Add::new(1, 2);
    let result = client.submit(&t).await?;
    let op = client.poll_result(&result, Duration::from_secs(10)).await;
    if let Err(e) = op {
        error!("fail to fetch result, {}", e);
    } else {
        info!("{:?}", op.unwrap());
    }
    info!("client done");
    sleep(Duration::from_secs(2)).await;
    Ok(())
}

async fn async_queue_server(server: Server) -> Result<(), String> {
    server.start(2).await?;
    info!("server done");
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let broker_url = "redis://127.0.0.1/";
    let queue = "asyncq:test_queue";
    let name = "async-queue";
    let aq = AsyncQueue::new(name, queue, broker_url).await;
    aq.register::<Add>().await.unwrap();

    let client = aq.client().await.unwrap();
    let server = aq.server().await.unwrap();

    let h = tokio::spawn(async_queue_server(server));
    async_queue_client(client).await.unwrap();
    let _ = h.await.unwrap();

    info!("main done");
}
