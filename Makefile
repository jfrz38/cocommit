CARGO ?= cargo
CARGO_DENY ?= cargo-deny
CARGO_DENY_VERSION := 0.20.2
ACTIONLINT ?= actionlint
SHELLCHECK ?= shellcheck

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

.PHONY: cargo-deny-install cargo-deny-version supply-chain-check
cargo-deny-install: ## install the pinned cargo-deny version
	$(CARGO) install cargo-deny --version $(CARGO_DENY_VERSION) --locked

cargo-deny-version: ## verify the installed cargo-deny version
	@$(CARGO_DENY) --version | grep -F "cargo-deny $(CARGO_DENY_VERSION)"

supply-chain-check: cargo-deny-version ## check dependency advisories, licenses, and sources
	CARGO_HOME="$(CURDIR)/target/cargo-home" $(CARGO_DENY) check advisories licenses sources

.PHONY: check check-portability check-workflows package-contents-check distribution-check release-check release-check-clean ci
check: fmt-check lint test build ## run all local quality checks

check-portability: fmt-check lint test build-release ## run the cross-platform quality suite

check-workflows: ## validate GitHub Actions workflows and embedded shell
	$(ACTIONLINT) -shellcheck=$(SHELLCHECK)
	bash .github/scripts/check-workflows.sh

package-contents-check: ## verify the files included in the crates.io package
	bash .github/scripts/check-package-contents.sh

distribution-check: package-contents-check ## verify distribution packaging scripts
	bash -n .github/scripts/check-package-contents.sh
	bash -n .github/scripts/package-distribution.sh
	bash -n .github/scripts/verify-distribution-archive.sh
	bash -n .github/scripts/verify-release-assets.sh

release-check: check ## verify the package can be published
	$(CARGO) publish --dry-run --locked --allow-dirty
	bash .github/scripts/check-package-contents.sh

release-check-clean: check ## verify a clean checkout can be published
	$(CARGO) publish --dry-run --locked
	bash .github/scripts/check-package-contents.sh

ci: check check-portability check-workflows supply-chain-check ## run local CI-equivalent checks
