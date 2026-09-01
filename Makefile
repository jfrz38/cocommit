CARGO ?= cargo

.DEFAULT_GOAL := help

.PHONY: help
help: ## show available development commands
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-16s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.PHONY: run build build-release
run: ## run the application
	$(CARGO) run --locked

build: ## build all targets
	$(CARGO) build --locked --all-targets --all-features

build-release: ## build all targets with the release profile
	$(CARGO) build --release --locked --all-targets --all-features

.PHONY: fmt fmt-check lint test
fmt: ## format Rust source files
	$(CARGO) fmt --all

fmt-check: ## verify Rust formatting
	$(CARGO) fmt --all -- --check

lint: ## run Clippy with warnings denied
	$(CARGO) clippy --locked --all-targets --all-features -- -D warnings

test: ## run all tests
	$(CARGO) test --locked --all-targets --all-features

.PHONY: check check-portability check-workflows release-check release-check-clean ci
check: fmt-check lint test build ## run all local quality checks

check-portability: fmt-check lint test build-release ## run the cross-platform quality suite

check-workflows: ## validate GitHub Actions workflows and embedded shell
	actionlint -shellcheck=shellcheck

release-check: check ## verify the package can be published
	$(CARGO) publish --dry-run --locked --allow-dirty

release-check-clean: check ## verify a clean checkout can be published
	$(CARGO) publish --dry-run --locked

ci: check check-portability check-workflows ## run local CI-equivalent checks
