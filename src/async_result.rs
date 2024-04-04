use crate::app::{signature::Signature, task::AQTask};

pub struct AsyncResult<T>
where
    T: AQTask,
{
    id: String,
    params: T::Params,
}

impl<T: AQTask> AsyncResult<T> {
    pub fn new(sig: &Signature<T>) -> AsyncResult<T> {
        AsyncResult {
            id: sig.get_id(),
            params: sig.get_params(),
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}
