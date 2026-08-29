CARGO ?= cargo

.DEFAULT_GOAL := help

.PHONY: help
help: ## show available development commands
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-16s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.PHONY: run build
run: ## run the application
	$(CARGO) run --locked

build: ## build all targets
	$(CARGO) build --locked --all-targets --all-features

.PHONY: fmt fmt-check lint test
fmt: ## format Rust source files
	$(CARGO) fmt --all

fmt-check: ## verify Rust formatting
	$(CARGO) fmt --all -- --check

lint: ## run Clippy with warnings denied
	$(CARGO) clippy --locked --all-targets --all-features -- -D warnings

test: ## run all tests
	$(CARGO) test --locked --all-targets --all-features

.PHONY: check release-check ci
check: fmt-check lint test build ## run all local quality checks

release-check: check ## verify the package can be published
	$(CARGO) publish --dry-run --locked --allow-dirty

ci: check ## run the deterministic CI checks
