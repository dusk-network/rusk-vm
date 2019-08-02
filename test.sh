cd ../test_contract/
cargo  build  --release --target wasm32-unknown-unknown 
cd ../core/
cargo test -- --nocapture
