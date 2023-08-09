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
    Echo {
        msg_id: usize,
        echo: String,
    },
    EchoOk {
        msg_id: usize,
        in_reply_to: usize,
        echo: String,
    },
    Error {
        in_reply_to: usize,
        code: usize,
        text: String,
    },
}

#[derive(Debug)]
struct Node {
    msg: Message<ResponseTypes>,
}

impl Response<ResponseTypes> for Node {
    type MessageImpl = Message<ResponseTypes>;

    fn serialize(&mut self, output: &mut impl Write) -> anyhow::Result<()> {
        let mut reply: Option<Self::MessageImpl> = None;

        match &self.msg.body {
            ResponseTypes::Init { msg_id, .. } => {
                reply = Some(Message {
                    src: self.msg.dest.clone(),
                    dest: self.msg.src.clone(),
                    body: ResponseTypes::InitOk {
                        in_reply_to: *msg_id,
                    },
                });
            }
            ResponseTypes::Echo { msg_id, echo } => {
                reply = Some(Message {
                    src: self.msg.dest.clone(),
                    dest: self.msg.src.clone(),
                    body: ResponseTypes::EchoOk {
                        msg_id: *msg_id,
                        in_reply_to: *msg_id,
                        echo: echo.clone(),
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

    for i in inputs {
        Node {
            msg: i.context("Couldn't deserialize STDIN")?,
        }
        .serialize(&mut stdout)?;
    }

    Ok(())
}