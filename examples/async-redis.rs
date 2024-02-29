use env_logger::Env;
use rust_async_queue::app::*;
use rust_async_queue::broker::*;
use serde::Deserialize;
use serde::Serialize;
use tokio::time::sleep;
use tokio::time::Duration;

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

async fn async_queue_server(server: Server) -> Result<(), String> {
    server.start(2).await?;
    println!("done");
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let broker_url = "redis://127.0.0.1/";
    let rbb = RedisBrokerBuilder::new(broker_url.to_owned());
    let broker = rbb.build(10).await.unwrap();
    let queue = "asyncq:test_queue".to_owned();
    let name = "async-queue".to_owned();
    let aq = AsyncQueue::new(name, queue, broker);
    aq.register::<Add>().await.unwrap();

    let client = aq.client().unwrap();
    let server = aq.server().unwrap();

    let h = tokio::spawn(async_queue_server(server));
    async_queue_client(client).await.unwrap();
    let _ = h.await.unwrap();

    println!("done");
}
