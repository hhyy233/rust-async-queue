use crate::broker::Broker;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tokio::time::{sleep, Timeout};

pub struct AsyncResult {
    id: String,
    broker: Arc<dyn Broker>,
}

impl AsyncResult {
    pub fn new(id: String, broker: Arc<dyn Broker>) -> AsyncResult {
        AsyncResult { id, broker }
    }

    pub fn poll(self, to: Duration) -> Timeout<impl Future<Output = Result<String, String>>> {
        let broker = self.broker.clone();
        let id = self.id.clone();
        let poll_fn = timeout(to, poll_fn(id, broker));
        return poll_fn;
    }
}

async fn poll_fn(id: String, broker: Arc<dyn Broker>) -> Result<String, String> {
    loop {
        println!("start polling");
        match broker.get(&id).await? {
            None => {
                sleep(Duration::from_millis(1000)).await;
            }
            Some(res) => {
                return Ok(res);
            }
        }
    }
}
