cd test_contracts/basic
cargo build --release --target wasm32-unknown-unknown
cd ../..
cd tools/printwat;
cargo run -- ../../test_contracts/basic/target/wasm32-unknown-unknown/release/test_contract.wasm
cd -
cargo test -- --nocapture
