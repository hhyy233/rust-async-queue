use super::{Broker, BrokerBuilder};
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::Client;

pub struct RedisBrokerBuilder {
    url: String,
}

#[async_trait]
impl BrokerBuilder for RedisBrokerBuilder {
    fn new(broker_url: String) -> Self
    where
        Self: Sized,
    {
        return RedisBrokerBuilder {
            url: broker_url.into(),
        };
    }

    async fn build(&self, _timeout: u32) -> Result<Box<dyn Broker>, String> {
        let client = Client::open(self.url.clone()).map_err(|e| e.to_string())?;
        let manager = client
            .get_tokio_connection_manager()
            .await
            .map_err(|e| e.to_string())?;
        return Ok(Box::new(RedisBroker { client, manager }));
    }
}

#[derive(Clone)]
pub struct RedisBroker {
    client: Client,
    manager: ConnectionManager,
}

#[async_trait]
impl Broker for RedisBroker {
    async fn get(&self, key: &String) -> Result<Option<String>, String> {
        let mut conn = self.manager.clone();
        let res = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
        return Ok(res);
    }

    async fn set(&self, key: &String, val: &String) -> Result<(), String> {
        let mut conn = self.manager.clone();
        println!("redis set {}, {}", key, val);
        redis::cmd("SET")
            .arg(key)
            .arg(val)
            .query_async(&mut conn)
            .await
            .map_err(|e| e.to_string())
    }

    async fn enqueue(&self, queue: &String, val: &String) -> Result<(), String> {
        let mut conn = self.manager.clone();
        redis::cmd("RPUSH")
            .arg(queue)
            .arg(val)
            .query_async(&mut conn)
            .await
            .map_err(|e| e.to_string())
    }

    async fn dequeue(&self, queue: &String) -> Result<Option<(String, String)>, String> {
        // we should create a new connection for blocking command.
        // https://github.com/redis-rs/redis-rs/issues/453
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| e.to_string())?;
        redis::cmd("BLPOP")
            .arg(queue)
            .arg(0) // no timeout
            .query_async(&mut conn)
            .await
            .map_err(|e| e.to_string())
    }
}
