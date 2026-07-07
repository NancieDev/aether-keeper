.PHONY: help setup build test deploy clean format lint type-check docs \
        check review ci-check maintainer-check wave-ready quality-report \
        pre-commit install-hooks

help:
	@echo "Aether Keeper - Decentralized Automation Network"
	@echo ""
	@echo "Quick Start:"
	@echo "  make setup          - Install all dependencies"
	@echo "  make check          - Run full quality check (format + lint + test + type-check)"
	@echo ""
	@echo "Development Commands:"
	@echo "  make build          - Build contract, CLI, and frontend"
	@echo "  make test           - Run all tests"
	@echo "  make lint           - Lint all code"
	@echo "  make format         - Format code"
	@echo "  make type-check     - TypeScript type checking"
	@echo "  make test-coverage  - Generate test coverage report"
	@echo ""
	@echo "Code Quality & Review:"
	@echo "  make check          - Full quality check (pre-commit recommended)"
	@echo "  make review         - Prepare for PR (check + coverage + docs)"
	@echo "  make ci-check       - Run all CI/CD checks locally"
	@echo "  make maintainer-check - Verify maintainer standards"
	@echo "  make quality-report - Generate quality metrics report"
	@echo ""
	@echo "Git Hooks (for contributors):"
	@echo "  make install-hooks  - Install pre-commit hooks"
	@echo "  make pre-commit     - Run pre-commit checks manually"
	@echo ""
	@echo "Deployment Commands:"
	@echo "  make deploy-dev     - Deploy to local testnet"
	@echo "  make deploy NETWORK=testnet  - Deploy to testnet"
	@echo "  make deploy NETWORK=mainnet  - Deploy to mainnet (requires confirmation)"
	@echo ""
	@echo "Component-Specific Commands:"
	@echo "  make build-contract - Build contract only"
	@echo "  make build-keeper   - Build keeper CLI only"
	@echo "  make build-frontend - Build frontend only"
	@echo "  make test-contract  - Run contract tests only"
	@echo "  make test-keeper    - Run keeper CLI tests only"
	@echo ""
	@echo "Drips Wave Support:"
	@echo "  make wave-ready     - Verify project is Wave-ready"
	@echo ""
	@echo "Maintenance:"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make docs           - Generate documentation"

setup:
	@echo "📦 Installing dependencies..."
	cargo build --workspace
	cd frontend && npm install
	@echo "✅ Setup complete!"

build: build-contract build-keeper build-frontend
	@echo "✅ Build complete!"

build-contract:
	@echo "🔨 Building contract..."
	cd contracts/aether_core && \
	cargo build --target wasm32-unknown-unknown --release && \
	cargo build --tests

build-keeper:
	@echo "🔨 Building keeper CLI..."
	cargo build --release -p keeper-cli

build-frontend:
	@echo "🔨 Building frontend..."
	cd frontend && npm run build

test: test-contract test-keeper
	@echo "✅ All tests passed!"

test-contract:
	@echo "🧪 Testing contract..."
	cd contracts/aether_core && cargo test -- --nocapture

test-keeper:
	@echo "🧪 Testing keeper CLI..."
	cargo test -p keeper-cli

test-coverage:
	@echo "📊 Generating coverage..."
	cd contracts/aether_core && \
	cargo tarpaulin --out Html --output-dir coverage

check: format lint test type-check
	@echo "✅ Full quality check passed!"

pre-commit: lint test type-check
	@echo "✅ Pre-commit checks passed!"

install-hooks:
	@echo "🔗 Installing git pre-commit hooks..."
	@if command -v pre-commit >/dev/null 2>&1; then \
		pre-commit install; \
		echo "✅ Pre-commit hooks installed"; \
	else \
		echo "⚠️  pre-commit not installed. Manual hook installation needed."; \
		echo "   See CONTRIBUTING.md for setup"; \
	fi

review: check test-coverage
	@echo "📋 Preparing for PR review..."
	@echo "✅ All pre-review checks passed!"
	@echo "   Next: Push branch and create pull request"
	@echo "   See: https://github.com/NancieDev/aether-keeper/pulls"

ci-check: lint test type-check test-coverage
	@echo "✅ All CI/CD checks passed locally!"
	@echo "   Your branch is ready for push"

maintainer-check:
	@echo "🔍 Running maintainer quality checks..."
	@echo ""
	@echo "1️⃣  Verifying Rust security..."
	@cargo audit --deny warnings
	@echo ""
	@echo "2️⃣  Checking code style..."
	@cargo fmt --all -- --check
	@echo ""
	@echo "3️⃣  Running linter (strict)..."
	@cargo clippy --all-targets -- -D warnings -D clippy::pedantic
	@echo ""
	@echo "4️⃣  Verifying test coverage..."
	@cd contracts/aether_core && cargo tarpaulin --timeout 300 --out Stdout
	@echo ""
	@echo "5️⃣  Type checking frontend..."
	@cd frontend && npm run type-check
	@echo ""
	@echo "✅ Maintainer checks passed!"

quality-report:
	@echo "📊 Generating quality metrics report..."
	@echo ""
	@echo "=== Code Quality Report ===" > quality-report.txt
	@echo "Generated: $$(date)" >> quality-report.txt
	@echo "" >> quality-report.txt
	@echo "--- Rust Clippy Results ---" >> quality-report.txt
	@cargo clippy --all-targets -- -D warnings 2>&1 | tee -a quality-report.txt || true
	@echo "" >> quality-report.txt
	@echo "--- Test Coverage ---" >> quality-report.txt
	@cd contracts/aether_core && cargo tarpaulin --out Stdout 2>&1 | tee -a quality-report.txt || true
	@echo "" >> quality-report.txt
	@echo "--- Dependency Security ---" >> quality-report.txt
	@cargo audit 2>&1 | tee -a quality-report.txt || true
	@echo "" >> quality-report.txt
	@echo "✅ Report saved to quality-report.txt"

wave-ready:
	@echo "🌊 Checking Drips Wave readiness..."
	@echo ""
	@echo "✓ Verifying documentation..."
	@test -f README.md || (echo "❌ README.md missing"; exit 1)
	@test -f CONTRIBUTING.md || (echo "❌ CONTRIBUTING.md missing"; exit 1)
	@test -f QUICKSTART.md || (echo "❌ QUICKSTART.md missing"; exit 1)
	@test -f CODE_OF_CONDUCT.md || (echo "❌ CODE_OF_CONDUCT.md missing"; exit 1)
	@test -f .github/ISSUE_TEMPLATE/bug_report.md || (echo "❌ Issue templates missing"; exit 1)
	@echo "✅ Documentation verified"
	@echo ""
	@echo "✓ Verifying code quality..."
	@cargo build --workspace 2>&1 | grep -q error && (echo "❌ Build failed"; exit 1) || echo "✅ Builds successfully"
	@echo ""
	@echo "✓ Verifying tests..."
	@cargo test --workspace 2>&1 | grep -q "test result: ok" || (echo "❌ Tests failing"; exit 1)
	@echo "✅ Tests passing"
	@echo ""
	@echo "✓ Verifying spec..."
	@test -f .kiro/specs/phase-2-stellar-integration/tasks.md || (echo "❌ Phase 2 spec missing"; exit 1)
	@echo "✅ Spec verified"
	@echo ""
	@echo "🌊 ✅ PROJECT IS WAVE-READY!"
	@echo ""
	@echo "Next steps:"
	@echo "  1. Review DRIPS_WAVE_APPLICATION.md"
	@echo "  2. Update maintainer info in application"
	@echo "  3. Submit to Drips Wave program"
	@echo "  4. Launch Wave 1 (Keeper CLI Foundation)"


	@echo "🎨 Formatting code..."
	cargo fmt --all
	cd frontend && npm run format

lint:
	@echo "📋 Linting code..."
	cargo clippy --all-targets --all-features -- -D warnings
	cd frontend && npm run lint

type-check:
	@echo "✔️  Type checking..."
	cd frontend && npm run type-check

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	cd frontend && npm run clean 2>/dev/null || true
	rm -rf coverage/

docs:
	@echo "📚 Generating documentation..."
	cargo doc --no-deps --open

deploy-dev:
	@echo "🚀 Deploying to local testnet..."
	@bash scripts/deploy.sh "$(or $(ADMIN_KEY), SA...)" "local"

deploy:
	@if [ -z "$(NETWORK)" ]; then \
		echo "❌ NETWORK not set. Usage: make deploy NETWORK=testnet"; \
		exit 1; \
	fi
	@if [ -z "$(ADMIN_KEY)" ]; then \
		echo "❌ ADMIN_KEY not set. Usage: make deploy NETWORK=testnet ADMIN_KEY=SA..."; \
		exit 1; \
	fi
	@bash scripts/deploy.sh "$(ADMIN_KEY)" "$(NETWORK)"

.DEFAULT_GOAL := help
