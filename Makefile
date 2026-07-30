.PHONY: build
build: ## build application
	cargo build

.PHONY: run
run: ## run application
	cargo run -- -k key-94e25400-f2ce-42a0-a9b5-44973aa372b9-integration_test.json

.PHONY: dryrun
dryrun: ## run application in --dry-run mode
	cargo run -- -k key-94e25400-f2ce-42a0-a9b5-44973aa372b9-integration_test.json --dry-run
