echo $1

cargo build --manifest-path=contracts/$1/Cargo.toml --release --target wasm32-unknown-unknown &&

cargo test $1 -- --nocapture

			
