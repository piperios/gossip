use anyhow::Context;
use gossip::{Message, Response};
use serde::{Deserialize, Serialize};
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
    Generate,
    GenereteOk {
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

    fn serialize(&mut self, output: &mut impl Write) -> anyhow::Result<()> {
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
                ResponseTypes::Generate => {
                    let guid = format!("{}-{}", self.node_id.clone(), self.id);
                    reply = Some(Message {
                        src: msg.dest.clone(),
                        dest: msg.src.clone(),
                        body: ResponseTypes::GenereteOk { guid },
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
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();

    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<ResponseTypes>>();

    let mut node = Node {
        node_id: String::new(),
        id: 1,
        msg: None,
    };

    for i in inputs {
        node.msg = Some(i.context("Couldn't deserialize STDIN")?);
        node.serialize(&mut stdout)?;
    }

    Ok(())
}
