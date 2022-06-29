all: ## Generate the optimized WASM for the contract given
	@cargo rustc \
		--manifest-path=./Cargo.toml \
		--release \
		--target wasm32-unknown-unknown \
		-Z build-std=core,alloc,panic_abort \
		-- -C link-args=-s
	        # -Z build-std-features=panic_immediate_abort \
