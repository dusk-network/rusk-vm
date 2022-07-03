help: ## Display this help screen
	@grep -h -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

all: test-contracts

test: test-contracts ## Run the contracts' tests
	cargo test --all-features

test-contracts: ## Build the test contracts
	$(MAKE) -C tests

test-session: ## test over sessions
	cargo run --manifest-path test_runner/Cargo.toml support
.PHONY: help all test test-contracts
