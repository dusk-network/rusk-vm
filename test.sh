cd test_contracts/basic
cargo build --release --target wasm32-unknown-unknown
cd ../..
cargo test -- --nocapture
