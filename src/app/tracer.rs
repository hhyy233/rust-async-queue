use async_trait::async_trait;

use super::{message::Message, task::AQTask};

#[async_trait]
pub trait TracerTrait: Send + Sync {
    /// Wraps the execution of a task, catching and logging errors and then running
    /// the appropriate post-execution functions.
    async fn run(&mut self) -> Result<String, String>;
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
    async fn run(&mut self) -> Result<String, String> {
        let res = self.task.run();
        serde_json::to_string(&res).map_err(|e| e.to_string())
    }
}

pub type TraceBuilderResult = Result<Box<dyn TracerTrait>, String>;

pub type TraceBuilder = Box<dyn Fn(Message) -> TraceBuilderResult + Send + Sync + 'static>;

pub fn build_tarce<T: AQTask + Send + Sync + 'static>(msg: Message) -> TraceBuilderResult {
    let payload = msg.get_payload();
    let params: T::Params = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let task: T = T::from_params(params);
    Ok(Box::new(Tracer::<T>::new(task)))
}
