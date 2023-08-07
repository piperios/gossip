use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<Body> {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

pub trait Response<Body> {
    type MessageImpl;
    fn serialize(&self) -> Option<Self::MessageImpl>;
}
