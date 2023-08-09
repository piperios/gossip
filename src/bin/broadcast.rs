use anyhow::Context;
use gossip::{Message, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;

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
        in_reply_to: usize,
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

        let msg = &self.msg.as_ref().unwrap();

        match &msg.body {
            ResponseTypes::Init { msg_id, .. } => {
                reply = Some(Message {
                    src: msg.dest.clone(),
                    dest: msg.src.clone(),
                    body: ResponseTypes::InitOk {
                        in_reply_to: *msg_id,
                    },
                });
            }
            ResponseTypes::Broadcast { message } => {
                self.msg_ids.push(*message);
                reply = Some(Message {
                    src: msg.dest.clone(),
                    dest: msg.src.clone(),
                    body: ResponseTypes::BroadcastOk,
                })
            }
            ResponseTypes::Topology(update) => {
                self.topology = update.clone();
                reply = Some(Message {
                    src: msg.dest.clone(),
                    dest: msg.src.clone(),
                    body: ResponseTypes::TopologyOk,
                })
            }
            ResponseTypes::Read => {
                reply = Some(Message {
                    src: msg.dest.clone(),
                    dest: msg.src.clone(),
                    body: ResponseTypes::ReadOk {
                        messages: self.msg_ids.clone(),
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

        if let Some(reply) = reply {
            serde_json::to_writer(&mut *output, &reply).context("Couldn't serialize reply")?;
            output.write_all(b"\n").context("Couldn't add newline")?;
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();

    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<ResponseTypes>>();

    let mut node = Node {
        msg: None,
        msg_ids: Vec::new(),
        topology: HashMap::new(),
    };

    for i in inputs {
        node.msg = Some(i.context("Couldn't deserialize STDIN")?);
        node.serialize(&mut stdout)?;
    }

    Ok(())
}
