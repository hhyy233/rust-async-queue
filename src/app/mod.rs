pub mod message;
mod signal;
pub mod signature;
pub mod task;
pub mod tracer;
mod worker;

use self::message::Message;
use self::signature::Signature;
use self::tracer::TracerTrait;
use tokio::time::timeout;
use tracing::debug;
use worker::Worker;

use crate::async_result::AsyncResult;
use crate::broker::Broker;
use crate::broker::BrokerBuilder;
use crate::broker::RedisBrokerBuilder;

use signal::*;
use std::collections::HashMap;
use std::sync::Arc;
use task::*;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

pub struct AsyncQueue {
    name: String,
    queue: String,
    broker_builder: Arc<dyn BrokerBuilder>,
    timeout: u32,
    task_builders: RwLock<HashMap<String, tracer::TraceBuilder>>,
}

impl AsyncQueue {
    pub async fn new(
        name: impl ToString,
        queue: impl ToString,
        broker_url: impl ToString,
    ) -> Arc<AsyncQueue> {
        let rbb = RedisBrokerBuilder::new(broker_url.to_string());

        // let broker = rbb.build(10).await.unwrap();
        Arc::new(AsyncQueue {
            name: name.to_string(),
            queue: queue.to_string(),
            broker_builder: Arc::from(rbb),
            timeout: 10,
            task_builders: RwLock::new(HashMap::new()),
        })
    }

    pub async fn client(self: &Arc<Self>) -> Result<Client, String> {
        let broker = self.broker_builder.build(self.timeout).await?;
        Ok(Client {
            queue: self.queue.clone(),
            broker: broker,
        })
    }

    pub async fn server(self: &Arc<Self>) -> Result<Server, String> {
        let broker = self.broker_builder.build(self.timeout).await?;
        Ok(Server {
            app: self.clone(),
            queue: self.queue.clone(),
            broker: broker,
            timeout: self.timeout,
            broker_builder: self.broker_builder.clone(),
        })
    }

    pub fn get_info(&self) -> String {
        self.name.clone()
    }

    pub async fn register<T: AQTask + 'static>(&self) -> Result<(), String> {
        let name = T::NAME;
        let mut task_builders = self.task_builders.write().await;
        if task_builders.contains_key(name) {
            return Err(format!("duplicate task {}", name));
        } else {
            task_builders.insert(name.into(), Box::new(tracer::build_tarce::<T>));
        }
        Ok(())
    }

    pub(crate) async fn get_tracer(
        self: &Arc<Self>,
        name: String,
        msg: Message,
    ) -> Result<Box<dyn TracerTrait>, String> {
        let task_builders = self.task_builders.read().await;
        if let Some(builder) = task_builders.get(&name) {
            let t = builder(msg)?;
            Ok(t)
        } else {
            Err(format!("task not found: {}", name))
        }
    }
}

pub struct Client {
    queue: String,
    broker: Box<dyn Broker>,
}

impl Client {
    pub async fn submit<T: AQTask>(&self, s: &Signature<T>) -> Result<AsyncResult<T>, String> {
        let msg = Message::try_from(s)?;
        let output = msg.serialize()?;
        self.broker.enqueue(&self.queue, &output).await?;

        Ok(AsyncResult::new(s))
    }

    pub async fn poll_result<T: AQTask>(
        &self,
        result: &AsyncResult<T>,
        to: Duration,
    ) -> Result<TaskReturn<T::Returns>, String> {
        let id = result.get_id();
        let poll_fn = timeout(to, poll_fn(id, &self.broker));
        match poll_fn.await {
            Ok(Ok(res)) => {
                let res: TaskReturn<T::Returns> = serde_json::from_str(&res)
                    .map_err(|e| format!("serde error: {}", e.to_string()));
                Ok(res)
            }
            Ok(Err(e)) => Err(format!("broker error: {}", e.to_string())),
            Err(e) => Err(e.to_string()),
        }
    }
}

async fn poll_fn(id: String, broker: &Box<dyn Broker>) -> Result<String, String> {
    loop {
        debug!("start polling");
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

pub struct Server {
    app: Arc<AsyncQueue>,
    queue: String,
    broker: Box<dyn Broker>,
    timeout: u32,
    broker_builder: Arc<dyn BrokerBuilder>,
}

impl Server {
    pub async fn start(&self, num: i32) -> Result<(), String> {
        info!("server start");
        // channel for tasks
        let (tx, rx) = async_channel::bounded(num as usize);
        // channel indicate if worker is free
        let (token_tx, token_rx) = mpsc::channel(num as usize);
        // channel for shutdown
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        for i in 0..num {
            let broker = self.broker_builder.build(self.timeout).await.unwrap();
            let w = Worker::new(i, broker, self.app.clone());

            let rx = rx.clone();
            let token_tx = token_tx.clone();
            let shutdown_tx = shutdown_tx.clone();
            tokio::spawn(async move { w.start(rx, token_tx, shutdown_tx).await });
        }
        drop(token_tx);
        drop(shutdown_tx);
        self.schedule(tx, token_rx).await?;

        // shutdown
        let _ = shutdown_rx.recv().await;
        info!("server closed");
        Ok(())
    }

    async fn schedule(
        &self,
        tx: async_channel::Sender<String>,
        mut token_rx: mpsc::Receiver<()>,
    ) -> Result<(), String> {
        // this is the flag indicate if we hold a token,
        // which means that there is a free worker waiting
        // and we are fine to poll a task from broker.
        let mut flag = false;
        let mut ender = Ender::new().map_err(|e| e.to_string())?;
        info!("scheduler start");
        loop {
            select! {
                _ = token_rx.recv(), if !flag => {
                    flag = true;
                },
                result = self.broker.dequeue(&self.queue), if flag => {
                    match result {
                        Ok(None) => {
                            let _ = sleep(Duration::from_secs(1));
                        }
                        Ok(Some((queue, task))) => {
                            flag = false;
                            info!("got from queue {}, {}", queue, task);
                            // TODO: error handle
                            match tx.send(task).await {
                                Ok(_) => {},
                                Err(e) => error!("error dispatch task {}", e),
                            }
                        },
                        Err(e) => {
                            error!("got error when dequeue from broker {}, {}", self.queue, e.to_string());
                        },
                    }
                },
                ending = ender.wait() => {
                    if let Ok(SigType::Interrupt) = ending {
                        warn!("got interrupt");
                    }
                    info!("Warm shutdown...");
                    break;
                }
            }
        }
        info!("scheduler closed");
        Ok(())
    }
}
