use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<Body> {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

pub trait Response<Body> {
    type MessageImpl;
    fn serialize<W>(&mut self, output: &mut W) -> anyhow::Result<()>
    where
        W: Write;
}
