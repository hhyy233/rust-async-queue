use crate::error::MsgError;

use super::{AQTask, Signature};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Message {
    id: String,
    name: String,
    payload: Vec<u8>,
}

impl Message {
    pub fn new(name: String, payload: Vec<u8>) -> Message {
        Message::new_with_id(Uuid::new_v4().to_string(), name, payload)
    }

    pub fn serialize(&self) -> Result<String, MsgError> {
        return serde_json::to_string(self).map_err(|e| e.into());
    }

    pub fn new_with_id(id: String, name: String, payload: Vec<u8>) -> Message {
        Message {
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

impl<T> TryFrom<&Signature<T>> for Message
where
    T: AQTask,
{
    type Error = MsgError;
    fn try_from(value: &Signature<T>) -> Result<Self, Self::Error> {
        let id = value.get_id();
        let name = value.name().to_string();
        let params = value.get_params();
        let payload = serde_json::to_vec(&params)?;
        Ok(Message::new_with_id(id, name, payload))
    }
}
