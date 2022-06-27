all: ## Generate the optimized WASM for the contract given
	@cargo rustc \
		--manifest-path=./Cargo.toml \
		--release \
		--target wasm32-unknown-unknown \
		-Z build-std=std,panic_abort \
        -Z build-std-features=panic_immediate_abort \
		-- -C link-args=-s
