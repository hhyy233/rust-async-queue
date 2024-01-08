use super::AQTask;
use uuid::Uuid;

#[derive(Clone)]
pub struct Signature<T>
where
    T: AQTask,
{
    id: String,
    params: T::Params,
}

impl<T> Signature<T>
where
    T: AQTask,
{
    /// Create a new `Signature` from task parameters.
    pub fn new(params: T::Params) -> Self {
        Self::new_with_id(Uuid::new_v4().to_string(), params)
    }
    pub fn new_with_id(id: String, params: T::Params) -> Self {
        Self { id, params }
    }
    pub fn get_id(&self) -> String {
        self.id.clone()
    }
    pub fn get_params(&self) -> T::Params {
        self.params.clone()
    }
    pub fn name(&self) -> &'static str {
        T::NAME
    }
}
