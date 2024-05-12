use rust_async_queue::{self, app::*};
use tokio::time::sleep;
use tokio::time::Duration;
use tracing::error;
use tracing::info;

// this is the target function we are using
#[rust_async_queue::task]
fn add(x: i32, y: i32) -> i32 {
    x + y
}

/*
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct addParam {
    x: i32,
    y: i32,
}

struct add {
    params: addParam,
}

impl task::AQTask for add {
    const NAME: &'static str = "add";
    type Params = addParam;
    type Returns = i32;
    fn run(&self) -> Self::Returns {
        add::_run(self.params.clone())
    }

    fn from_params(params: Self::Params) -> Self {
        Self { params }
    }
}

impl add {
    fn new(x: i32, y: i32) -> signature::Signature<Self> {
        signature::Signature::<add>::new(addParam { x, y })
    }
    fn _run(params: addParam) -> i32 {
        let x = params.x;
        let y = params.y;
        x + y
    }
}
*/

async fn async_queue_client(client: Client) -> Result<(), String> {
    let t = add::new(1, 2);
    let result = client.submit(&t).await?;
    let op = client.poll_result(&result, Duration::from_secs(10)).await;
    match op {
        Ok(res) => {
            info!("got result {:?}", res);
            assert_eq!(Ok(3), res, "want {:?}, got {:?}", 3, res);
        }
        Err(e) => error!("fail to fetch result, {}", e),
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
    aq.register::<add>().await.unwrap();

    let client = aq.client().await.unwrap();
    let server = aq.server().await.unwrap();

    let h = tokio::spawn(async_queue_server(server));
    async_queue_client(client).await.unwrap();
    let _ = h.await.unwrap();

    info!("main done");
}
