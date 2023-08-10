use anyhow::Context;
use gossip::{Message, Payload, Response};
use serde::{Deserialize, Serialize};
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
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
    Error {
        code: usize,
        text: String,
    },
}

#[derive(Debug)]
struct Node {
    msg: Option<Message<ResponseTypes>>,
}

impl Response<ResponseTypes> for Node {
    type MessageImpl = Message<ResponseTypes>;

    fn serialize<W>(&mut self, output: &mut W) -> anyhow::Result<()>
    where
        W: Write,
    {
        let mut payload: Option<Payload<ResponseTypes>> = None;
        if let Some(ref msg) = &self.msg {
            match &msg.body.payload {
                ResponseTypes::Init { .. } => {
                    payload = Some(Payload {
                        msg_id: msg.body.msg_id,
                        in_reply_to: msg.body.msg_id,
                        payload: ResponseTypes::InitOk,
                    });
                }
                ResponseTypes::Echo { echo } => {
                    payload = Some(Payload {
                        msg_id: msg.body.msg_id,
                        in_reply_to: msg.body.msg_id,
                        payload: ResponseTypes::EchoOk { echo: echo.clone() },
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

        if let Some(payload) = payload {
            let msg = self.msg.as_ref().unwrap();
            let reply = Message {
                src: msg.dest.clone(),
                dest: msg.src.clone(),
                body: payload,
            };

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

    let mut node = Node { msg: None };
    node.run_loop(stdin, stdout)?;

    Ok(())
}
