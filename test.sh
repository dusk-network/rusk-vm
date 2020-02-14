#!/usr/bin/env bash

echo $1

WASM_BUILD_RUSTFLAGS='-C link-arg=-s' cargo build --manifest-path=tests/contracts/$1/wasm/Cargo.toml --release --target wasm32-unknown-unknown &&

cargo test $1 -- --nocapture


