#![allow(dead_code)]
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("redis pool error: {0}")]
    PoolError(#[from] PoolError),
}

#[derive(Error, Debug)]
pub enum PoolError {
    #[error("cannot creat redis client: {0}")]
    RedisClientError(redis::RedisError),
    // #[error("cannot creat redis pool: {0}")]
    // RedisPoolError(mobc::Error<redis::RedisError>),
    #[error("cannot run command: {0}")]
    RedisCMDError(redis::RedisError),
    #[error("cannot parse value: {0}")]
    RedisTypeError(redis::RedisError),
}
