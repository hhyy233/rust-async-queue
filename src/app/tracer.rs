use crate::error::TracerError;

use super::{message::Message, task::AQTask};
use async_trait::async_trait;

#[async_trait]
pub trait TracerTrait: Send + Sync {
    /// Wraps the execution of a task, catching and logging errors and then running
    /// the appropriate post-execution functions.
    async fn run(&mut self) -> Result<String, TracerError>;
}

pub struct Tracer<T>
where
    T: AQTask,
{
    task: T,
}

impl<T> Tracer<T>
where
    T: AQTask,
{
    pub fn new(task: T) -> Self {
        Tracer { task }
    }
}

#[async_trait]
impl<T> TracerTrait for Tracer<T>
where
    T: AQTask,
{
    async fn run(&mut self) -> Result<String, TracerError> {
        let res = self.task.run();
        serde_json::to_string(&res).map_err(|e| e.into())
    }
}

pub type TraceBuilderResult = Result<Box<dyn TracerTrait>, TracerError>;

pub type TraceBuilder = Box<dyn Fn(Message) -> TraceBuilderResult + Send + Sync + 'static>;

pub fn build_trace<T: AQTask + Send + Sync + 'static>(msg: Message) -> TraceBuilderResult {
    let payload = msg.get_payload();
    let params: T::Params = serde_json::from_slice(payload)?;
    let task: T = T::from_params(params);
    Ok(Box::new(Tracer::<T>::new(task)))
}
