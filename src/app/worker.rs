use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::error;
use tracing::info;

use crate::app::message::Message;
use crate::broker::Broker;

use super::AsyncQueue;

pub(crate) struct Worker {
    id: i32,
    broker: Box<dyn Broker>,
    app: Arc<AsyncQueue>,
}

impl Worker {
    pub fn new(i: i32, broker: Box<dyn Broker>, app: Arc<AsyncQueue>) -> Self {
        Worker {
            id: i,
            broker: broker,
            app: app,
        }
    }

    pub async fn start(
        &self,
        rx: async_channel::Receiver<String>,
        tx: mpsc::Sender<()>,
        _: mpsc::Sender<()>,
    ) {
        let idx = self.id;
        info!(worker = idx, "start");
        loop {
            if rx.is_closed() {
                break;
            }
            if let Err(e) = tx.send(()).await {
                error!(worker = idx, "fail to give out token, {}", e.to_string());
                let _ = sleep(Duration::from_secs(1));
                continue;
            }
            let res = rx.recv().await;
            match res {
                Ok(val) => {
                    info!(worker = idx, "got {}", val);
                    if let Err(e) = self.handle(val).await {
                        error!(worker = idx, "got error handl task {}", e);
                    }
                }
                Err(e) => {
                    if rx.is_closed() {
                        break;
                    }
                    error!(worker = idx, "error {}", e.to_string());
                }
            }
        }
        info!(worker = idx, "stopped");
    }

    async fn handle(&self, val: String) -> Result<(), String> {
        let idx = self.id;

        let msg: Message = serde_json::from_str(&val[..]).map_err(|e| e.to_string())?;
        let id = msg.get_id();
        let name = msg.get_name();

        let payload = String::from_utf8_lossy(msg.get_payload());
        info!(worker = idx, "got task {}, {}", id, payload);

        let result = self.handle_message(name, msg).await?;
        self.broker.set(&id, &result).await?;
        info!(worker = idx, "write result to {}, {}", id, result);
        Ok(())
    }

    async fn handle_message(&self, name: String, msg: Message) -> Result<String, String> {
        let mut tracer = self.app.get_tracer(name, msg).await?;
        tracer.run().await
    }
}
