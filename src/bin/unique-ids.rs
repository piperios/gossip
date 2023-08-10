use anyhow::Context;
use gossip::{Message, Response};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum ResponseTypes {
    Init {
        msg_id: usize,
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {
        in_reply_to: usize,
    },
    Generate {
        msg_id: usize,
    },
    GenerateOk {
        msg_id: usize,
        in_reply_to: usize,
        #[serde(rename = "id")]
        guid: String,
    },
    Error {
        in_reply_to: usize,
        code: usize,
        text: String,
    },
}

#[derive(Debug)]
struct Node {
    node_id: String,
    id: usize,
    msg: Option<Message<ResponseTypes>>,
}

impl Response<ResponseTypes> for Node {
    type MessageImpl = Message<ResponseTypes>;

    fn serialize<W>(&mut self, output: &mut W) -> anyhow::Result<()>
    where
        W: Write,
    {
        let mut reply: Option<Self::MessageImpl> = None;

        if let Some(ref msg) = &self.msg {
            match &msg.body {
                ResponseTypes::Init {
                    msg_id, node_id, ..
                } => {
                    self.node_id = node_id.clone();
                    reply = Some(Message {
                        src: msg.dest.clone(),
                        dest: msg.src.clone(),
                        body: ResponseTypes::InitOk {
                            in_reply_to: *msg_id,
                        },
                    });
                }
                ResponseTypes::Generate { msg_id } => {
                    let guid = format!("{}-{}", self.node_id.clone(), self.id);
                    self.id += 1;
                    reply = Some(Message {
                        src: msg.dest.clone(),
                        dest: msg.src.clone(),
                        body: ResponseTypes::GenerateOk {
                            msg_id: *msg_id + 1,
                            in_reply_to: *msg_id,
                            guid,
                        },
                    });
                }
                ResponseTypes::Error { text, .. } => {
                    eprintln!("{}", text);
                }
                _ => {
                    eprintln!("{}", "Impossible input!");
                }
            };
        }

        if let Some(reply) = reply {
            serde_json::to_writer(&mut *output, &reply).context("Couldn't serialize reply")?;
            output.write_all(b"\n").context("Couldn't add newline")?;
        }

        Ok(())
    }

    fn run_loop<R, W>(&mut self, input: R, mut output: W) -> anyhow::Result<()>
    where
        R: Read,
        W: Write,
    {
        let inputs =
            serde_json::Deserializer::from_reader(input).into_iter::<Message<ResponseTypes>>();

        for i in inputs {
            self.msg = Some(i.context("Couldn't deserialize STDIN")?);
            self.serialize(&mut output)?;
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    let stdout = std::io::stdout().lock();

    let mut node = Node {
        node_id: String::new(),
        id: 1,
        msg: None,
    };

    node.run_loop(stdin, stdout)?;

    Ok(())
}
