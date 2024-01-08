use serde::{Deserialize, Serialize};

pub type TaskReturn<R> = Result<R, String>;

pub trait AQTask: Send + Sync {
    const NAME: &'static str;
    type Params: Send + Sync + Clone + Serialize + for<'de> Deserialize<'de>;
    type Returns: Send + Sync + Clone + Serialize + for<'de> Deserialize<'de>;
    fn run(&self) -> TaskReturn<Self::Returns>;
    fn from_params(params: Self::Params) -> Self;
}
