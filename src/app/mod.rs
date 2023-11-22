pub mod async_result;
pub mod task;

use std::sync::Arc;

use crate::broker::Broker;
use async_channel::{bounded, Receiver, Sender};
use async_result::*;

use log::{debug, error, info, warn};
use task::*;
use tokio::select;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

pub struct AsyncQueue {
    name: String,
    queue: String,
    broker: Arc<dyn Broker>,
}

impl AsyncQueue {
    pub fn new(name: String, queue: String, broker: Box<dyn Broker>) -> AsyncQueue {
        AsyncQueue {
            name: name,
            queue: queue,
            broker: Arc::from(broker),
        }
    }

    pub fn client(&self) -> Result<Client, String> {
        Ok(Client {
            queue: self.queue.clone(),
            broker: self.broker.clone(),
        })
    }

    pub fn server(&self) -> Result<Server, String> {
        Ok(Server {
            queue: self.queue.clone(),
            broker: self.broker.clone(),
        })
    }

    pub fn get_info(&self) -> String {
        self.name.clone()
    }
}

pub struct Client {
    queue: String,
    broker: Arc<dyn Broker>,
}

impl Client {
    pub async fn submit(&self, t: &Task) -> Result<AsyncResult, String> {
        let output = serde_json::to_string(t).map_err(|e| e.to_string())?;
        self.broker.enqueue(&self.queue, &output).await?;
        Ok(AsyncResult::new(t.get_id(), self.broker.clone()))
    }
}

pub struct Server {
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

        let task: Task = serde_json::from_str(&val[..]).map_err(|e| e.to_string())?;
        let task_id = task.get_id();
        let task_payload = String::from_utf8_lossy(task.get_payload());
        info!("worker {}, got task {}, {}", idx, task_id, task_payload);

        let res = "done".to_owned();
        self.broker.set(&task_id, &res).await?;
        info!("worker {}, write result to {}, {}", idx, task_id, res);
        Ok(())
    }
}

#[allow(unused)]
enum SigType {
    /// Equivalent to SIGINT on unix systems.
    Interrupt,
    /// Equivalent to SIGTERM on unix systems.
    Terminate,
}

#[cfg(windows)]
struct Ender;

#[cfg(windows)]
impl Ender {
    fn new() -> Result<Self, std::io::Error> {
        Ok(Ender)
    }

    async fn wait(&mut self) -> Result<SigType, std::io::Error> {
        info!("listening on signal");
        tokio::signal::ctrl_c().await?;

        Ok(SigType::Interrupt)
    }
}
