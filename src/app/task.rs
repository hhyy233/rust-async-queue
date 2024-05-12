use serde::{Deserialize, Serialize};
use std::fmt;

pub type TaskReturn<R> = Result<R, String>;

pub trait AQTask: Send + Sync {
    const NAME: &'static str;
    type Params: Send + Sync + Clone + Serialize + for<'de> Deserialize<'de>;
    type Returns: Send + Sync + Clone + Serialize + for<'de> Deserialize<'de> + fmt::Debug;
    fn run(&self) -> Self::Returns;
    fn from_params(params: Self::Params) -> Self;
}
