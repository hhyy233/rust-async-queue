use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Task {
    id: String,
    name: String,
    payload: Vec<u8>,
}

impl Task {
    pub fn new(name: String, payload: Vec<u8>) -> Task {
        Task::new_with_id(Uuid::new_v4().to_string(), name, payload)
    }

    pub fn new_with_id(id: String, name: String, payload: Vec<u8>) -> Task {
        Task {
            id: id,
            name,
            payload,
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_payload(&self) -> &Vec<u8> {
        &self.payload
    }
}
