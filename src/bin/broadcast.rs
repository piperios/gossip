use anyhow::Context;
use gossip::{Message, Payload, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum ResponseTypes {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
    Topology(HashMap<String, Vec<String>>),
    TopologyOk,
    Error {
        code: usize,
        text: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct TopologyUpdate {
    node_id: String,
    neighbors: Vec<String>,
}

#[derive(Debug)]
struct Node {
    msg: Option<Message<ResponseTypes>>,
    msg_ids: Vec<usize>,
    topology: HashMap<String, Vec<String>>,
}

impl Response<ResponseTypes> for Node {
    type MessageImpl = Message<ResponseTypes>;

    fn serialize<W>(&mut self, output: &mut W) -> anyhow::Result<()>
    where
        W: Write,
    {
        let mut reply: Option<Self::MessageImpl> = None;

        if let Some(ref msg) = &self.msg {
            match &msg.body.payload {
                ResponseTypes::Init { .. } => {
                    reply = Some(Message {
                        src: msg.dest.clone(),
                        dest: msg.src.clone(),
                        body: Payload {
                            msg_id: Some(msg.body.msg_id.unwrap() + 1),
                            in_reply_to: msg.body.msg_id,
                            payload: ResponseTypes::InitOk,
                        },
                    });
                }
                ResponseTypes::Broadcast { message } => {
                    self.msg_ids.push(*message);
                    reply = Some(Message {
                        src: msg.dest.clone(),
                        dest: msg.src.clone(),
                        body: Payload {
                            msg_id: Some(msg.body.msg_id.unwrap() + 1),
                            in_reply_to: msg.body.msg_id,
                            payload: ResponseTypes::BroadcastOk,
                        },
                    })
                }
                ResponseTypes::Topology(update) => {
                    self.topology = update.clone();
                    reply = Some(Message {
                        src: msg.dest.clone(),
                        dest: msg.src.clone(),
                        body: Payload {
                            msg_id: Some(msg.body.msg_id.unwrap() + 1),
                            in_reply_to: msg.body.msg_id,
                            payload: ResponseTypes::TopologyOk,
                        },
                    })
                }
                ResponseTypes::Read => {
                    reply = Some(Message {
                        src: msg.dest.clone(),
                        dest: msg.src.clone(),
                        body: Payload {
                            msg_id: Some(msg.body.msg_id.unwrap() + 1),
                            in_reply_to: msg.body.msg_id,
                            payload: ResponseTypes::ReadOk {
                                messages: self.msg_ids.clone(),
                            },
                        },
                    })
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
        msg: None,
        msg_ids: Vec::new(),
        topology: HashMap::new(),
    };
    node.run_loop(stdin, stdout)?;

    Ok(())
}