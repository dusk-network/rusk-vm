SUBDIRS := $(wildcard ./tests/contracts/*/.)

help: ## Display this help screen
	@grep -h -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

all: $(SUBDIRS)

test: $(SUBDIRS) ## Run the contracts' tests
	cargo test --features "persistence"

gasmonitor: 

$(SUBDIRS):
	$(MAKE) -C $@

.PHONY: help all test $(SUBDIRS)
