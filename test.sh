#!/usr/bin/env bash

echo $1

WASM_BUILD_RUSTFLAGS='-C link-arg=-s' cargo build --manifest-path=contracts/$1/Cargo.toml --release --target wasm32-unknown-unknown &&

cargo test $1 -- --nocapture


