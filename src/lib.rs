use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

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
    fn run_loop<R, W>(&mut self, input: R, output: W) -> anyhow::Result<()>
    where
        R: Read,
        W: Write;
}
