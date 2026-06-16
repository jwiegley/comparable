# Comparable -- developer task runner.
#
# Most targets assume the Nix dev shell is active (automatically via direnv, or
# `nix develop`).  Miri and fuzzing need the nightly shell:
#
#     nix develop .#nightly --command make miri
#     nix develop .#nightly --command make fuzz
#
# Run `make help` for a categorized list of targets.

SHELL := bash
CARGO_TARGET_DIR ?= target
TARGET = $(HOME)/dfinity/fixtures/rs/comparable

.PHONY: all help gen-lib build check test doc docs docs-open \
	fmt fmt-check lint clippy nix-lint shell-lint \
	coverage coverage-lcov coverage-check coverage-baseline \
	bench perf-check perf-baseline perf-compare \
	miri fuzz fuzz-smoke audit ci clean publish

all: ci

help: ## Show this help
	@grep -hE '^[a-zA-Z0-9_/.-]+:.*?## ' $(MAKEFILE_LIST) \
		| sed -E 's/:[^#]*## /\t/' | sort | column -t -s "$$(printf '\t')"

# --- Code generation -------------------------------------------------------
#
# lib.rs is committed; regenerate it only after editing README.md or
# lib.rs.in.  rustfmt normalizes the result to the project style.

gen-lib: comparable/src/lib.rs ## Regenerate lib.rs from README.md + lib.rs.in

comparable/src/lib.rs: README.md comparable/src/lib.rs.in
	{ sed 's%^%//! %' README.md; cat comparable/src/lib.rs.in; } > $@
	rustfmt $@

# --- Build & test ----------------------------------------------------------

build: ## Build the workspace (warnings denied via [lints])
	cargo build --workspace --all-targets

test: ## Run unit, integration and doc tests
	cargo test --workspace

check: ## Type-check the workspace
	cargo check --workspace --all-targets

# --- Formatting ------------------------------------------------------------

fmt: ## Format Rust, Nix and shell sources
	cargo fmt --all
	if git ls-files '*.nix' | grep -q .; then nixpkgs-fmt $$(git ls-files '*.nix'); fi
	if git ls-files '*.sh' | grep -q .; then shfmt -w $$(git ls-files '*.sh'); fi

fmt-check: ## Check formatting of Rust, Nix and shell sources
	cargo fmt --all -- --check
	nixpkgs-fmt --check $$(git ls-files '*.nix')
	shfmt -d $$(git ls-files '*.sh')

# --- Linting ---------------------------------------------------------------

lint: clippy nix-lint shell-lint ## Run all linters

clippy: ## Lint Rust with clippy (warnings denied)
	cargo clippy --workspace --all-targets -- -D warnings

nix-lint: ## Lint Nix with statix
	statix check

shell-lint: ## Lint shell scripts with shellcheck
	shellcheck $$(git ls-files '*.sh')

# --- Documentation ---------------------------------------------------------

doc docs: ## Build API docs (warnings denied) and run doctests
	cargo test --doc --workspace
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

docs-open: docs ## Build and open API docs
	cargo doc --workspace --no-deps --open

# --- Coverage --------------------------------------------------------------

coverage: ## Generate an HTML coverage report
	cargo llvm-cov --workspace --html
	@echo "HTML report: $(CARGO_TARGET_DIR)/llvm-cov/html/index.html"

coverage-lcov: ## Generate an lcov.info coverage report
	cargo llvm-cov --workspace --lcov --output-path lcov.info

coverage-check: ## Fail if coverage drops below .coverage-baseline
	./scripts/check-coverage.sh

coverage-baseline: ## Record current coverage as the new baseline
	cargo llvm-cov --workspace --no-report
	cargo llvm-cov report --json | jq '.data[0].totals.lines.percent' > .coverage-baseline
	@echo "wrote .coverage-baseline: $$(cat .coverage-baseline)"

# --- Benchmarks / performance ----------------------------------------------

bench: ## Run criterion benchmarks (performance report)
	cargo bench --bench comparison

perf-baseline: ## Save current benchmark medians as the local baseline
	cargo bench --bench comparison -- --noplot
	./scripts/check-perf.sh --save

perf-check: ## Fail if benchmarks regress >5% vs the local baseline
	cargo bench --bench comparison -- --noplot
	./scripts/check-perf.sh

perf-compare: ## Compare benchmark runs with critcmp
	critcmp new

# --- Memory / UB checking --------------------------------------------------

miri: ## Run the test suite under Miri (needs nightly shell)
	# Isolate HOME/CARGO_HOME/RUSTUP_HOME so cargo-miri uses the Nix nightly
	# toolchain instead of an ambient rustup proxy.
	tmp=$$(mktemp -d); \
	env -u CARGO_HOME -u RUSTUP_HOME -u RUSTUP_TOOLCHAIN HOME="$$tmp" PROPTEST_CASES=8 MIRIFLAGS="-Zmiri-disable-isolation" cargo miri test --workspace; \
	rc=$$?; rm -rf "$$tmp"; exit $$rc

# --- Fuzzing ---------------------------------------------------------------

fuzz: ## Fuzz the comparison engine (needs nightly shell)
	cargo fuzz run compare

fuzz-smoke: ## Short fuzz run for CI (needs nightly shell)
	cargo fuzz run compare -- -max_total_time=60 -runs=300000

# --- Misc ------------------------------------------------------------------

audit: ## Audit dependencies for security advisories
	cargo audit

ci: fmt-check lint build test doc ## Fast checks (pre-commit / nix flake check)

clean: ## Remove generated artifacts
	rm -f lcov.info
	find . -name '*~' -print0 | xargs -0 /bin/rm -f

publish: ## (maintainer) rsync working tree to the local mirror
	@rsync -av --delete --delete-excluded \
		--exclude 'Makefile' \
		--exclude '*~' --exclude '/Cargo.*' --exclude 'default.nix' \
		--exclude '.git' --exclude '.direnv' --exclude '.envrc*' \
		./ $(TARGET)/
	@echo ""
	@echo "Delta files have been published to $(TARGET)"
