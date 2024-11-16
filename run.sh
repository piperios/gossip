#!/bin/bash

# Build specific binary
cargo build --release --bin $1

if [ "$1" == "echo" ]; then
    ./maelstrom/maelstrom test -w echo --bin ./target/release/echo --node-count 1 --time-limit 10
elif [ "$1" == "unique_ids" ]; then
    ./maelstrom/maelstrom test -w unique-ids --bin ./target/release/unique-ids --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition
elif [ "$1" == "broadcast" ]; then
    ./maelstrom/maelstrom test -w broadcast --bin ./target/release/broadcast --node-count 1 --time-limit 20 --rate 10
fi