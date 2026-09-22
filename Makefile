.PHONY: build
build: ## build application
	cargo build

.PHONY: delete
delete: ## run application
	cargo run -- purge-inactive-identities -k key-94e25400-f2ce-42a0-a9b5-44973aa372b9-integration_test.json

.PHONY: display
display: ## run application in display mode without deleting
	cargo run -- list-inactive-identities -k key-94e25400-f2ce-42a0-a9b5-44973aa372b9-integration_test.json

.PHONY: lint
lint: ## run linter
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: update
update: ## run cargo upgrade && cargo update
	cargo upgrade && cargo update

.PHONY: test
test: ## run tests
	cargo nextest run
