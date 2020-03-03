test: hello factorial
	rustup target add wasm32-unknown-unknown
	cargo test

hello: tests/contracts/hello/wasm/src/lib.rs
	WASM_BUILD_RUSTFLAGS='-C link-arg=-s' cargo build --manifest-path=tests/contracts/$@/wasm/Cargo.toml --release --target wasm32-unknown-unknown

factorial: tests/contracts/factorial/wasm/src/lib.rs
	WASM_BUILD_RUSTFLAGS='-C link-arg=-s' cargo build --manifest-path=tests/contracts/$@/wasm/Cargo.toml --release --target wasm32-unknown-unknown
