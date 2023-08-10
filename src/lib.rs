use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<T> {
    pub src: String,
    pub dest: String,
    pub body: Payload<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload<T> {
    pub msg_id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: T,
}

pub trait Response<Body> {
    type MessageImpl;
    fn serialize<W>(&mut self, output: &mut W) -> anyhow::Result<()>
    where
        W: Write;
    fn run_loop<R, W>(&mut self, input: R, output: W) -> anyhow::Result<()>
    where
        R: Read,
        W: Write;
}
