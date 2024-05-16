use std::io;

use thiserror::Error;
use tokio::time::error::Elapsed;

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("deserialization error: {0}")]
    DeserdeError(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum QueueError {
    #[error("broker error: {0}")]
    BrokerError(#[from] BrokerError),

    #[error("duplicate task {0}")]
    DuplicateTask(String),
}

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("broker error: {0}")]
    BrokerError(#[from] BrokerError),

    #[error("tracer error: {0}")]
    TracerError(#[from] TracerError),

    #[error("serialization error: {0}")]
    ProtocolError(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("message error: {0}")]
    MsgError(#[from] MsgError),

    #[error("broker error: {0}")]
    BrokerError(#[from] BrokerError),

    #[error("io error: {0}")]
    IOError(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("message error: {0}")]
    MsgError(#[from] MsgError),

    #[error("broker error: {0}")]
    BrokerError(#[from] BrokerError),

    #[error("timeout")]
    Timeout(#[from] Elapsed),
}

#[derive(Error, Debug)]
pub enum MsgError {
    #[error("serialization error: {0}")]
    ProtocolError(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum TracerError {
    #[error("serialization error: {0}")]
    ProtocolError(#[from] serde_json::Error),

    #[error("cannot found task {0}")]
    TaskNotFound(String),
}

#[derive(Error, Debug)]
pub enum BrokerError {
    #[error("redis error: {0}")]
    RedisError(#[from] redis::RedisError),
}
