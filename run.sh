#!/usr/bin/bash

cargo build --release

# maelstrom test -w echo --bin ./target/release/echo --node-count 1 --time-limit 10
# maelstrom test -w unique-ids --bin ./target/release/unique-ids --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition
maelstrom test -w broadcast --bin ./target/release/broadcast --node-count 1 --time-limit 20 --rate 10
