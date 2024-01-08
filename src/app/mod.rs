pub mod async_result;
pub mod message;
mod signal;
pub mod signature;
pub mod task;
pub mod tracer;
use self::message::Message;
use self::signature::Signature;
use self::tracer::TracerTrait;
use crate::broker::Broker;
use async_channel::{bounded, Receiver, Sender};
use async_result::*;
use log::{error, info, warn};
use signal::*;
use std::collections::HashMap;
use std::sync::Arc;
use task::*;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

pub struct AsyncQueue {
    name: String,
    queue: String,
    broker: Arc<dyn Broker>,
    task_builders: RwLock<HashMap<String, tracer::TraceBuilder>>,
}

impl AsyncQueue {
    pub fn new(name: String, queue: String, broker: Box<dyn Broker>) -> Arc<AsyncQueue> {
        Arc::new(AsyncQueue {
            name: name,
            queue: queue,
            broker: Arc::from(broker),
            task_builders: RwLock::new(HashMap::new()),
        })
    }

    pub fn client(self: &Arc<Self>) -> Result<Client, String> {
        Ok(Client {
            queue: self.queue.clone(),
            broker: self.broker.clone(),
        })
    }

    pub fn server(self: &Arc<Self>) -> Result<Server, String> {
        Ok(Server {
            app: self.clone(),
            queue: self.queue.clone(),
            broker: self.broker.clone(),
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

    pub async fn get_tracer(
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
    broker: Arc<dyn Broker>,
}

impl Client {
    pub async fn submit<T: AQTask>(&self, s: &Signature<T>) -> Result<AsyncResult, String> {
        let msg = Message::try_from(s)?;
        let output = msg.serialize()?;
        self.broker.enqueue(&self.queue, &output).await?;
        Ok(AsyncResult::new(msg.get_id(), self.broker.clone()))
    }
}

pub struct Server {
    app: Arc<AsyncQueue>,
    queue: String,
    broker: Arc<dyn Broker>,
}

impl Server {
    pub async fn start(&self, num: i32) -> Result<(), String> {
        info!("server start");
        // channel for tasks
        let (tx, rx) = bounded(num as usize);
        // channel indicate if worker is free
        let (token_tx, token_rx) = mpsc::channel(num as usize);
        // channel for shutdown
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        for i in 0..num {
            let w = Worker {
                id: i,
                broker: self.broker.clone(),
                app: self.app.clone(),
            };
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
        tx: Sender<String>,
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
                            let _ = tx.send(task).await;
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

pub struct Worker {
    id: i32,
    broker: Arc<dyn Broker>,
    app: Arc<AsyncQueue>,
}

impl Worker {
    async fn start(&self, rx: Receiver<String>, tx: mpsc::Sender<()>, _: mpsc::Sender<()>) {
        let idx = self.id;
        info!("worker {} start", idx);
        loop {
            if rx.is_closed() {
                break;
            }
            if let Err(e) = tx.send(()).await {
                error!("worker {}, fail to give out token, {}", idx, e.to_string());
                let _ = sleep(Duration::from_secs(1));
                continue;
            }
            let res = rx.recv().await;
            match res {
                Ok(val) => {
                    info!("worker {}, got {}", idx, val);
                    if let Err(e) = self.handle(val).await {
                        error!("worker {}, got error handl task {}", idx, e);
                    }
                }
                Err(e) => {
                    if rx.is_closed() {
                        break;
                    }
                    error!("worker {}, error {}", idx, e.to_string());
                }
            }
        }
        info!("worker {} stopped", idx);
    }
    async fn handle(&self, val: String) -> Result<(), String> {
        let idx = self.id;

        let msg: Message = serde_json::from_str(&val[..]).map_err(|e| e.to_string())?;
        let id = msg.get_id();
        let name = msg.get_name();

        let payload = String::from_utf8_lossy(msg.get_payload());
        info!("worker {}, got task {}, {}", idx, id, payload);

        let result = self.handle_message(name, msg).await?;
        self.broker.set(&id, &result).await?;
        info!("worker {}, write result to {}, {}", idx, id, result);
        Ok(())
    }

    async fn handle_message(&self, name: String, msg: Message) -> Result<String, String> {
        let mut tracer = self.app.get_tracer(name, msg).await?;
        tracer.run().await
    }
}
