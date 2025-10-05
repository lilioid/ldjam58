#!/usr/bin/env bash
D=$(realpath $(dirname $(dirname $0)))

exec docker run -i --rm \
  -v ldjam58-cargo-cache:/usr/local/cargo \
  -v $D:/usr/local/src/ldjam58 docker.io/rust \
  <<EOF
  cd /usr/local/src/ldjam58
  rustup target add wasm32-unknown-unknown
  cargo install wasm-bindgen-cli
  cargo install wasm-opt
  cargo install --git https://github.com/TheBevyFlock/bevy_cli --locked bevy_cli
  bevy build --release web --bundle -v
EOF
