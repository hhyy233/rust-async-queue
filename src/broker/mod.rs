pub mod redis;

pub use self::redis::RedisBrokerBuilder;
use crate::error::BrokerError;
use async_trait::async_trait;

#[async_trait]
pub trait BrokerBuilder: Send + Sync {
    /// Create a new `BrokerBuilder`.
    fn new(broker_url: String) -> Self
    where
        Self: Sized;

    /// Construct the `Broker` with the given configuration.
    async fn build(&self, timeout: u32) -> Result<Box<dyn Broker>, BrokerError>;
}

#[async_trait]
pub trait Broker: Send + Sync {
    async fn get(&self, key: &String) -> Result<Option<String>, BrokerError>;
    async fn set(&self, key: &String, val: &String) -> Result<(), BrokerError>;
    async fn enqueue(&self, queue: &String, val: &String) -> Result<(), BrokerError>;
    async fn dequeue(&self, queue: &String) -> Result<Option<(String, String)>, BrokerError>;
}
