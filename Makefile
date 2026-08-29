# XDL — Developer Makefile
#
# One entry point for every build, test, lint, and consistency check the
# project has. Run `make help` for the full list.
#
#   Surface                 Build                 Test
#   ──────────────────────  ────────────────────  ──────────────────────
#   Rust workspace          make build            make test
#   xdl-cli (interpreter)   make build-cli        make smoke
#   xdl-lsp (language srv)  make build-lsp        —
#   xdl-matlab (transpiler) make build-matlab     make test-matlab
#   xdl-desktop-viewer      make check-desktop    —
#   xdl-chart-viewer        make build-viewer     —
#   xdl-charts (ECharts)    make build-charts     —
#   VS Code extension       make build-vscode     make lint-vscode
#   XDL language suite      make build-cli        make suite
#   Docs (Rust API + Jekyll) make docs            —
#
# Aggregates:
#   make build         Rust workspace (release)
#   make check         Fast type-check (workspace + desktop-viewer + vscode)
#   make test          Rust workspace tests + xdl-matlab
#   make test-all      Tests + language suite + consistency scripts
#   make lint          clippy + formatting check
#   make ci            Mirror the GitHub CI gate locally
#   make parity        All consistency scripts (builtin, version, docs-links)

.PHONY: help help-surfaces doctor \
        build build-cli build-lsp build-matlab build-viewer build-charts build-vscode \
        check check-workspace check-desktop check-vscode \
        test test-workspace test-matlab test-all \
        smoke suite suite-survey \
        lint lint-rust fmt fmt-check \
        parity parity-builtin parity-version parity-docs \
        docs docs-rust docs-jekyll \
        coverage clean

# Ensure ~/.cargo/bin is on PATH (npm/rustup shadowing on some systems)
export PATH := $(HOME)/.cargo/bin:$(PATH)

# ── Toolchain ─────────────────────────────────────────────────────────────────
CARGO      ?= cargo
NPM        ?= npm
XDL_BIN    ?= ./target/release/xdl
VSCODE_DIR := vscode-xdl

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*##' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*## "}; {printf "  \033[36m%-22s\033[0m %s\n", $$1, $$2}'

help-surfaces: ## Print the per-surface build/test matrix (from the header)
	@sed -n '4,20p' $(MAKEFILE_LIST)

doctor: ## Verify the development environment
	@echo "Checking XDL development environment..."
	@echo ""
	@printf "  %-22s" "Rust:" && (rustc --version 2>/dev/null || echo "MISSING — run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh")
	@printf "  %-22s" "Cargo:" && (cargo --version 2>/dev/null || echo "MISSING")
	@printf "  %-22s" "Node.js:" && (node --version 2>/dev/null || echo "not installed (needed for vscode-xdl) — https://nodejs.org/")
	@printf "  %-22s" "npm:" && (npm --version 2>/dev/null || echo "not installed (needed for vscode-xdl)")
	@printf "  %-22s" "Git:" && (git --version 2>/dev/null || echo "MISSING")
	@printf "  %-22s" "python3:" && (python3 --version 2>/dev/null || echo "not installed (needed for docs anchor check)")
	@printf "  %-22s" "cargo-llvm-cov:" && (cargo llvm-cov --version 2>/dev/null || echo "not installed (needed for coverage) — cargo install cargo-llvm-cov")
	@printf "  %-22s" "Tauri CLI:" && (cargo tauri --version 2>/dev/null || echo "not installed (needed for viewer dev) — cargo install tauri-cli")
	@echo ""
	@echo "Required: Rust, Cargo, Git"
	@echo "Optional: Node.js + npm (VS Code ext), python3 (docs checks), cargo-llvm-cov (coverage), Tauri CLI (viewers)"
ifeq ($(shell uname -s),Linux)
	@echo ""
	@echo "Linux — checking Tauri system dependencies..."
	@for dep in libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev; do \
		printf "  %-36s" "$$dep:" && (dpkg -s $$dep 2>/dev/null | grep -q "ok installed" && echo "OK" || echo "MISSING"); \
	done
endif

# ══════════════════════════════════════════════════════════════════════════════
# BUILD — Rust workspace + per-crate targets
# ══════════════════════════════════════════════════════════════════════════════

build: ## Build the full Rust workspace (release)
	$(CARGO) build --workspace --release
	@echo ""
	@ls -lh target/release/xdl 2>/dev/null || true
	@echo ""
	@echo "Binary: target/release/xdl"

build-cli: ## Build the XDL interpreter (release)
	$(CARGO) build --release -p xdl-cli
	@ls -lh $(XDL_BIN)

build-lsp: ## Build the XDL language server (release)
	$(CARGO) build --release -p xdl-lsp
	@ls -lh target/release/xdl-lsp

build-matlab: ## Build the MATLAB transpiler crate
	$(CARGO) build -p xdl-matlab

build-viewer: ## Build the Tauri chart viewer
	$(CARGO) build -p xdl-chart-viewer

build-charts: ## Build the ECharts HTML generation crate
	$(CARGO) build -p xdl-charts

# ── VS Code extension ─────────────────────────────────────────────────────────

$(VSCODE_DIR)/node_modules:
	cd $(VSCODE_DIR) && $(NPM) install --no-audit --no-fund

build-vscode: $(VSCODE_DIR)/node_modules ## Compile the VS Code extension (tsc)
	cd $(VSCODE_DIR) && $(NPM) run compile

# ══════════════════════════════════════════════════════════════════════════════
# CHECK — fast type-checks (no codegen)
# ══════════════════════════════════════════════════════════════════════════════

check: check-workspace check-desktop ## Fast type-check (Rust workspace + desktop-viewer)

check-workspace: ## Type-check the Rust workspace
	$(CARGO) check --workspace --all-targets

check-desktop: ## Type-check xdl-desktop-viewer (outside the workspace)
	$(CARGO) check --manifest-path xdl-desktop-viewer/Cargo.toml

check-vscode: $(VSCODE_DIR)/node_modules ## Type-check the VS Code extension (tsc --noEmit)
	cd $(VSCODE_DIR) && npx tsc --noEmit

# ══════════════════════════════════════════════════════════════════════════════
# TEST — Rust unit tests + XDL language suite
# ══════════════════════════════════════════════════════════════════════════════

test: test-workspace test-matlab ## Run all Rust tests (workspace + xdl-matlab)

test-workspace: ## Run Rust workspace tests (--no-fail-fast)
	$(CARGO) test --workspace --all-targets --no-fail-fast

test-matlab: ## Run xdl-matlab tests (not covered by --workspace test target)
	$(CARGO) test -p xdl-matlab --all-targets --no-fail-fast

test-all: test suite parity ## Everything: Rust tests + language suite + consistency checks
	@echo ""
	@echo "✓ All tests and consistency checks passed."

# ── Smoke test the interpreter binary ─────────────────────────────────────────

smoke: build-cli ## Smoke-test the interpreter: --version + a findgen call
	@$(XDL_BIN) --version
	@$(XDL_BIN) -e 'print, findgen(5)'
	@echo "✓ Interpreter smoke test passed."

# ── XDL language suite (tests/*.xdl via allowlist ratchet) ────────────────────

suite: build-cli ## Run the XDL language suite (allowlist ratchet)
	@bash scripts/run-xdl-suite.sh

suite-survey: build-cli ## Survey every .xdl script (informational, never gates)
	@bash scripts/run-xdl-suite.sh --survey

# ══════════════════════════════════════════════════════════════════════════════
# LINT — clippy + formatting
# ══════════════════════════════════════════════════════════════════════════════

lint: lint-rust fmt-check ## Run clippy + check formatting

lint-rust: ## Run clippy on the workspace (-D warnings)
	$(CARGO) clippy --workspace --all-targets -- -D warnings

lint-vscode: $(VSCODE_DIR)/node_modules ## Lint the VS Code extension (eslint)
	cd $(VSCODE_DIR) && $(NPM) run lint

fmt: ## Format all Rust code (cargo fmt)
	$(CARGO) fmt --all

fmt-check: ## Check Rust formatting without modifying
	$(CARGO) fmt --all -- --check

# ══════════════════════════════════════════════════════════════════════════════
# PARITY — consistency scripts (traps a build cannot catch)
# ══════════════════════════════════════════════════════════════════════════════
#
# These guard invariants the compiler does not enforce: builtin name lists
# drifting between stdlib/LSP/grammar, version numbers disagreeing across
# Cargo.toml/tauri/package.json, and broken doc links. See AGENTS.md ->
# Traps a Build Cannot Catch.

parity: parity-builtin parity-version parity-docs ## Run all consistency checks

parity-builtin: ## Check stdlib dispatch ↔ LSP symbols ↔ TextMate grammar
	@bash scripts/check-builtin-parity.sh

parity-version: ## Check version sync across Cargo.toml / tauri.conf.json / package.json
	@bash scripts/check-version-sync.sh

parity-docs: ## Check relative links + heading anchors in docs
	@bash scripts/check-docs-links.sh

# ══════════════════════════════════════════════════════════════════════════════
# DOCS — Rust API docs + Jekyll site
# ══════════════════════════════════════════════════════════════════════════════

docs: docs-rust ## Build Rust API documentation (cargo doc --no-deps)

docs-rust: ## Build Rust API docs for the workspace
	$(CARGO) doc --workspace --no-deps

# Jekyll site is published by .github/workflows/pages.yml. Local preview:
#   cd docs && bundle exec jekyll serve

# ══════════════════════════════════════════════════════════════════════════════
# COVERAGE — cargo llvm-cov
# ══════════════════════════════════════════════════════════════════════════════

coverage: ## Generate code coverage report (LCOV format)
	@command -v cargo-llvm-cov >/dev/null 2>&1 || { \
		echo "✗ cargo-llvm-cov not found — install: cargo install cargo-llvm-cov"; exit 1; }
	cargo llvm-cov --workspace --all-targets --lcov --output-path coverage.lcov
	cargo llvm-cov -p xdl-matlab --all-targets --lcov --output-path coverage-matlab.lcov
	@echo "✓ Coverage reports: coverage.lcov, coverage-matlab.lcov"
	@echo "  View HTML locally: cargo llvm-cov --workspace --html"

# ══════════════════════════════════════════════════════════════════════════════
# CI — mirror the GitHub CI gate locally
# ══════════════════════════════════════════════════════════════════════════════
#
# Mirrors .github/workflows/ci.yml: consistency, test, lint, language-suite, docs.
# Run this before pushing.

ci: ## Run the same checks CI does (parity + lint + test + suite + docs)
	@echo "── Consistency checks ─────────────────────────────────"
	$(MAKE) parity
	@echo ""
	@echo "── Formatting + clippy ────────────────────────────────"
	$(MAKE) lint
	@echo ""
	@echo "── Rust workspace tests + xdl-matlab ──────────────────"
	$(MAKE) test
	@echo ""
	@echo "── xdl-desktop-viewer (outside workspace) ─────────────"
	$(MAKE) check-desktop
	@echo ""
	@echo "── XDL language suite (allowlist ratchet) ─────────────"
	$(MAKE) suite
	@echo ""
	@echo "── Documentation build ────────────────────────────────"
	$(MAKE) docs
	@echo ""
	@echo "✓ Local CI gate passed."

# ══════════════════════════════════════════════════════════════════════════════
# CLEAN
# ══════════════════════════════════════════════════════════════════════════════

clean: ## Remove Rust build artifacts
	$(CARGO) clean
	rm -f coverage.lcov coverage-matlab.lcov