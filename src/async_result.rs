use std::marker::PhantomData;

use crate::app::{signature::Signature, task::AQTask};

pub struct AsyncResult<T>
where
    T: AQTask,
{
    id: String,
    phantom: PhantomData<T>,
}

impl<T: AQTask> AsyncResult<T> {
    pub fn new(sig: &Signature<T>) -> AsyncResult<T> {
        AsyncResult {
            id: sig.get_id(),
            phantom: PhantomData,
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}
