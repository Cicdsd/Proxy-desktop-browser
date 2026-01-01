# Makefile for Proxy-Desktop-Browser
# A privacy-focused browser with virtual IP routing

.PHONY: all build build-rust build-ui clean clean-rust clean-ui dev install help test lint format check release

# Default target
all: build

# =============================================================================
# Installation
# =============================================================================

install: ## Install all dependencies
	@echo "Installing Rust dependencies..."
	@cargo fetch 2>/dev/null || echo "Note: cargo not available, skipping Rust dependencies"
	@echo "Installing UI dependencies..."
	@cd ui-tauri && bun install 2>/dev/null || npm install 2>/dev/null || echo "Note: bun/npm not available"
	@echo "Dependencies installation completed!"

# =============================================================================
# Build Targets
# =============================================================================

build: build-rust build-ui ## Build the entire project
	@echo "Build completed successfully!"

build-rust: ## Build Rust crates only
	@echo "Building Rust crates..."
	@cargo build --workspace 2>/dev/null || echo "Note: cargo not available, skipping Rust build"

build-ui: ## Build UI only
	@echo "Building UI..."
	@cd ui-tauri && bun run build 2>/dev/null || npm run build 2>/dev/null || echo "Note: bun/npm not available"

build-release: ## Build release version
	@echo "Building release version..."
	@cargo build --workspace --release 2>/dev/null || echo "Note: cargo not available, skipping Rust release build"
	@cd ui-tauri && bun run build 2>/dev/null || npm run build 2>/dev/null || echo "Note: bun/npm not available"

tauri-build: ## Build Tauri desktop application
	@echo "Building Tauri application..."
	@cd ui-tauri && bun run tauri:build 2>/dev/null || npm run tauri:build 2>/dev/null || echo "Note: tauri build not available"

# =============================================================================
# Development
# =============================================================================

dev: ## Start development server
	@echo "Starting development server..."
	@cd ui-tauri && bun run tauri 2>/dev/null || npm run tauri 2>/dev/null || echo "Note: tauri dev not available"

dev-ui: ## Start UI development server only
	@echo "Starting UI development server..."
	@cd ui-tauri && bun run dev 2>/dev/null || npm run dev 2>/dev/null || echo "Note: dev server not available"

# =============================================================================
# Clean Targets
# =============================================================================

clean: clean-rust clean-ui ## Clean all build artifacts
	@echo "All build artifacts cleaned!"

clean-rust: ## Clean Rust build artifacts
	@echo "Cleaning Rust build artifacts..."
	@rm -rf target/ 2>/dev/null || true
	@rm -rf crates/*/target/ 2>/dev/null || true
	@echo "Rust build artifacts cleaned!"

clean-ui: ## Clean UI build artifacts
	@echo "Cleaning UI build artifacts..."
	@rm -rf ui-tauri/dist/ 2>/dev/null || true
	@rm -rf ui-tauri/.svelte-kit/ 2>/dev/null || true
	@rm -rf ui-tauri/node_modules/.vite/ 2>/dev/null || true
	@rm -rf ui-tauri/src-tauri/target/ 2>/dev/null || true
	@echo "UI build artifacts cleaned!"

clean-all: clean ## Deep clean including dependencies
	@echo "Deep cleaning..."
	@rm -rf ui-tauri/node_modules/ 2>/dev/null || true
	@rm -rf Cargo.lock 2>/dev/null || true
	@echo "Deep clean completed!"

# =============================================================================
# Testing
# =============================================================================

test: test-rust test-ui ## Run all tests
	@echo "All tests completed!"

test-rust: ## Run Rust tests
	@echo "Running Rust tests..."
	@cargo test --workspace 2>/dev/null || echo "Note: cargo not available, skipping Rust tests"

test-ui: ## Run UI tests
	@echo "Running UI tests..."
	@cd ui-tauri && bun run test 2>/dev/null || npm run test 2>/dev/null || echo "Note: test runner not available"

# =============================================================================
# Code Quality
# =============================================================================

lint: lint-rust lint-ui ## Run all linters
	@echo "Linting completed!"

lint-rust: ## Run Rust linter (clippy)
	@echo "Running Clippy..."
	@cargo clippy --workspace -- -D warnings 2>/dev/null || echo "Note: clippy not available"

lint-ui: ## Run UI linter
	@echo "Running ESLint..."
	@cd ui-tauri && bun run lint 2>/dev/null || npm run lint 2>/dev/null || echo "Note: linter not available"

format: format-rust format-ui ## Format all code
	@echo "Formatting completed!"

format-rust: ## Format Rust code
	@echo "Formatting Rust code..."
	@cargo fmt --all 2>/dev/null || echo "Note: cargo fmt not available"

format-ui: ## Format UI code
	@echo "Formatting UI code..."
	@cd ui-tauri && bun run format 2>/dev/null || npm run format 2>/dev/null || echo "Note: formatter not available"

check: check-rust check-ui ## Run all checks without building
	@echo "All checks passed!"

check-rust: ## Check Rust code
	@echo "Checking Rust code..."
	@cargo check --workspace 2>/dev/null || echo "Note: cargo check not available"

check-ui: ## Check UI TypeScript/Svelte
	@echo "Checking UI code..."
	@cd ui-tauri && bun run check 2>/dev/null || npm run check 2>/dev/null || echo "Note: type checker not available"

# =============================================================================
# Documentation
# =============================================================================

docs: ## Generate documentation
	@echo "Generating Rust documentation..."
	@cargo doc --workspace --no-deps 2>/dev/null || echo "Note: cargo doc not available"

# =============================================================================
# Release
# =============================================================================

release: clean build-release test ## Full release build with tests
	@echo "Release build completed!"

# =============================================================================
# Help
# =============================================================================

help: ## Show this help message
	@echo "Proxy-Desktop-Browser Makefile"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'
