use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

static mut NODE_ID: usize = 1;

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Body {
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
    Generate,
    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
    Error {
        in_reply_to: usize,
        code: usize,
        text: String,
    },
}

fn serialize_message(input: Message, output: &mut StdoutLock) -> anyhow::Result<()> {
    let mut reply: Option<Message> = None;

    match input.body {
        Body::Init { msg_id, .. } => {
            reply = Some(Message {
                src: input.dest,
                dest: input.src,
                body: Body::InitOk {
                    in_reply_to: msg_id,
                },
            });
        }
        Body::Echo { msg_id, echo } => {
            reply = Some(Message {
                src: input.dest,
                dest: input.src,
                body: Body::EchoOk {
                    msg_id: msg_id + 1,
                    in_reply_to: msg_id,
                    echo,
                },
            });
        }
        Body::Generate => unsafe {
            let guid = format!("{}-{}", input.dest, NODE_ID);
            reply = Some(Message {
                src: input.dest,
                dest: input.src,
                body: Body::GenerateOk { guid },
            });
            NODE_ID += 1;
        },
        Body::Error { text, .. } => {
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

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();

    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    for i in inputs {
        let input = i.context("Couldn't deserialize STDIN")?;
        serialize_message(input, &mut stdout)?;
    }

    Ok(())
}
