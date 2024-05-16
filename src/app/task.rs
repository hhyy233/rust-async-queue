use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::TaskError;

pub type TaskReturn<R> = Result<R, TaskError>;

pub trait AQTask: Send + Sync {
    const NAME: &'static str;
    type Params: Send + Sync + Clone + Serialize + for<'de> Deserialize<'de>;
    type Returns: Send + Sync + Clone + Serialize + for<'de> Deserialize<'de> + fmt::Debug;
    fn run(&self) -> Self::Returns;
    fn from_params(params: Self::Params) -> Self;
}
