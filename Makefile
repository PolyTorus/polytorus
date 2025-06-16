# Makefile for Polytorus Kani Verification

.PHONY: kani-install kani-setup kani-verify kani-clean kani-quick kani-crypto kani-blockchain kani-modular kani-security kani-performance kani-watch kani-report pre-commit ci-verify ci-verify-quick kani-dev kani-list kani-check dep-check kani-ci fmt fmt-check clippy docker docker-dev docker-clean help

# Colors for output
BLUE := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[1;33m
RED := \033[0;31m
NC := \033[0m # No Color

# Default target
help:
	@echo "$(BLUE)Polytorus Kani Verification Makefile$(NC)"
	@echo ""	@echo "Available targets:"
	@echo "  $(GREEN)kani-install$(NC)     - Install Kani verifier"
	@echo "  $(GREEN)kani-setup$(NC)       - Setup Kani for this project"
	@echo "  $(GREEN)kani-verify$(NC)      - Run all Kani verifications"
	@echo "  $(GREEN)kani-quick$(NC)       - Run quick verification subset"
	@echo "  $(GREEN)kani-crypto$(NC)      - Run cryptographic verifications only"
	@echo "  $(GREEN)kani-blockchain$(NC)  - Run blockchain verifications only"
	@echo "  $(GREEN)kani-modular$(NC)     - Run modular architecture verifications only"
	@echo "  $(GREEN)kani-security$(NC)    - Run security-focused verifications"
	@echo "  $(GREEN)kani-performance$(NC) - Run performance-oriented verifications"
	@echo "  $(GREEN)kani-clean$(NC)       - Clean verification results"
	@echo "  $(GREEN)pre-commit$(NC)       - Run pre-commit checks (fmt + clippy)"	@echo "  $(GREEN)fmt$(NC)              - Format code with rustfmt"
	@echo "  $(GREEN)fmt-check$(NC)        - Check code formatting"
	@echo "  $(GREEN)clippy$(NC)           - Run clippy linter"	@echo "  $(GREEN)ci-verify$(NC)        - Run full CI verification pipeline"
	@echo "  $(GREEN)ci-verify-quick$(NC)  - Run quick CI verification (no Kani)"
	@echo "  $(GREEN)docker$(NC)           - Build Docker image"
	@echo "  $(GREEN)docker-dev$(NC)       - Start development environment"
	@echo "  $(GREEN)docker-clean$(NC)     - Clean Docker resources"
	@echo "  $(GREEN)deps-check$(NC)       - Check dependency status"
	@echo "  $(GREEN)security-audit$(NC)   - Run security audit"
	@echo "  $(GREEN)docs$(NC)             - Build and open documentation"
	@echo "  $(GREEN)help$(NC)             - Show this help message"

# Install Kani
kani-install:
	@echo "$(BLUE)Installing Kani verifier...$(NC)"
	cargo install --locked kani-verifier
	cargo kani setup

# Setup Kani for this project
kani-setup:
	@echo "$(BLUE)Setting up Kani for Polytorus...$(NC)"
	@if ! command -v kani &> /dev/null; then \
		echo "$(RED)Kani not found. Installing...$(NC)"; \
		$(MAKE) kani-install; \
	fi
	@echo "$(GREEN)Kani setup complete!$(NC)"

# Run all verifications
kani-verify: kani-setup
	@echo "$(BLUE)Running complete Kani verification suite...$(NC)"
	cd kani-verification && bash ./run_verification.sh

# Run quick verification (subset for development)
kani-quick: kani-setup
	@echo "$(BLUE)Running quick Kani verification...$(NC)"
	@mkdir -p verification_results
	cd kani-verification && cargo kani --harness verify_basic_arithmetic
	cd kani-verification && cargo kani --harness verify_encryption_type_determination
	cd kani-verification && cargo kani --harness verify_block_hash_consistency
	cd kani-verification && cargo kani --harness verify_modular_architecture_structure
	@echo "$(GREEN)Quick verification complete!$(NC)"

# Run cryptographic verifications only
kani-crypto: kani-setup
	@echo "$(BLUE)Running cryptographic verifications...$(NC)"
	cd kani-verification && cargo kani --harness verify_encryption_type_determination
	cd kani-verification && cargo kani --harness verify_transaction_integrity
	cd kani-verification && cargo kani --harness verify_signature_properties
	cd kani-verification && cargo kani --harness verify_public_key_format
	cd kani-verification && cargo kani --harness verify_hash_computation
	@echo "$(GREEN)Cryptographic verification complete!$(NC)"

# Run blockchain verifications only
kani-blockchain: kani-setup
	@echo "$(BLUE)Running blockchain verifications...$(NC)"
	cd kani-verification && cargo kani --harness verify_block_hash_consistency
	cd kani-verification && cargo kani --harness verify_blockchain_integrity
	cd kani-verification && cargo kani --harness verify_difficulty_adjustment
	cd kani-verification && cargo kani --harness verify_invalid_block_rejection
	@echo "$(GREEN)Blockchain verification complete!$(NC)"

# Run modular architecture verifications only
kani-modular: kani-setup
	@echo "$(BLUE)Running modular architecture verifications...$(NC)"
	cd kani-verification && cargo kani --harness verify_modular_architecture_structure
	cd kani-verification && cargo kani --harness verify_layer_communication
	cd kani-verification && cargo kani --harness verify_invalid_communication_rejection
	cd kani-verification && cargo kani --harness verify_layer_state_update
	cd kani-verification && cargo kani --harness verify_synchronization_mechanism
	@echo "$(GREEN)Modular architecture verification complete!$(NC)"

# Run security-focused verifications
kani-security: kani-setup
	@echo "$(BLUE)Running security-focused verifications...$(NC)"
	cd kani-verification && cargo kani --harness verify_array_bounds
	cd kani-verification && cargo kani --harness verify_transaction_value_bounds
	cd kani-verification && cargo kani --harness verify_invalid_block_rejection
	cd kani-verification && cargo kani --harness verify_invalid_communication_rejection
	@echo "$(GREEN)Security verification complete!$(NC)"

# Performance testing with Kani
kani-performance: kani-setup
	@echo "$(BLUE)Running performance-oriented verifications...$(NC)"
	cd kani-verification && timeout 120 cargo kani --harness verify_queue_operations
	cd kani-verification && timeout 120 cargo kani --harness verify_hash_determinism
	cd kani-verification && timeout 120 cargo kani --harness verify_synchronization_mechanism
	@echo "$(GREEN)Performance verification complete!$(NC)"

# Watch mode for continuous verification during development
kani-watch: kani-setup
	@echo "$(BLUE)Starting Kani watch mode...$(NC)"
	@echo "Will re-run verification when files change..."
	@while true; do \
		$(MAKE) kani-quick; \
		echo "$(YELLOW)Waiting for file changes... (Ctrl+C to stop)$(NC)"; \
		sleep 10; \
	done

# Generate verification report
kani-report: kani-verify
	@echo "$(BLUE)Generating verification report...$(NC)"
	@mkdir -p docs/verification
	@if [ -f kani-verification/kani_results/summary.md ]; then \
		cp kani-verification/kani_results/summary.md docs/verification/latest-report.md; \
		echo "$(GREEN)Verification report generated at docs/verification/latest-report.md$(NC)"; \
	else \
		echo "$(RED)No verification results found. Run 'make kani-verify' first.$(NC)"; \
	fi

# Development workflow - quick check before commit
pre-commit: fmt clippy
	@echo "$(GREEN)Pre-commit verification passed!$(NC)"

# Format code
fmt:
	@echo "$(BLUE)Running cargo fmt...$(NC)"
	cargo fmt --all
	@echo "$(GREEN)Code formatting completed!$(NC)"

# Check formatting
fmt-check:
	@echo "$(BLUE)Checking code formatting...$(NC)"
	cargo fmt --all -- --check

# Run clippy
clippy:
	@echo "$(BLUE)Running cargo clippy...$(NC)"
	cargo clippy --all-targets --all-features -- -W clippy::all
	@echo "$(GREEN)Clippy checks passed!$(NC)"

# Run clippy with strict rules (for CI)
clippy-strict:
	@echo "$(BLUE)Running strict cargo clippy...$(NC)"
	cargo clippy --all-targets --all-features -- -D warnings -W clippy::all
	@echo "$(GREEN)Strict clippy checks passed!$(NC)"

# CI workflow - comprehensive verification
ci-verify: fmt-check clippy kani-verify kani-report
	@echo "$(GREEN)CI verification workflow complete!$(NC)"

# CI workflow without Kani (faster)
ci-verify-quick: fmt-check clippy
	@echo "$(GREEN)Quick CI verification workflow complete!$(NC)"

# Docker management
docker:
	@echo "$(BLUE)Building Docker image...$(NC)"
	docker build -f Dockerfile.optimized -t polytorus:latest .

docker-dev:
	@echo "$(BLUE)Starting development environment...$(NC)"
	docker-compose -f docker-compose.dev.yml up -d

docker-clean:
	@echo "$(BLUE)Cleaning Docker resources...$(NC)"
	docker-compose -f docker-compose.dev.yml down -v
	docker system prune -f

# Dependency management
deps-check:
	@echo "$(BLUE)Checking dependencies...$(NC)"
	cargo outdated
	cargo audit

deps-update:
	@echo "$(BLUE)Updating dependencies...$(NC)"
	cargo update

# Security checks
security-audit:
	@echo "$(BLUE)Running security audit...$(NC)"
	cargo audit
	cargo deny check

# Documentation
docs:
	@echo "$(BLUE)Building documentation...$(NC)"
	cargo doc --all-features --no-deps --open

docs-serve:
	@echo "$(BLUE)Serving documentation...$(NC)"
	cargo doc --all-features --no-deps
	python3 -m http.server 8080 -d target/doc

# Development targets
.PHONY: kani-dev kani-list kani-check

# Development verification (faster, smaller bounds)
kani-dev: kani-setup
	@echo "$(BLUE)Running development verification (fast)...$(NC)"
	@mkdir -p verification_results
	cargo kani --harness verify_encryption_type_determination --solver-option="--bounds-check=off"
	cargo kani --harness verify_layer_state_transitions --solver-option="--bounds-check=off"
	@echo "$(GREEN)Development verification complete!$(NC)"

# List all available harnesses
kani-list:
	@echo "$(BLUE)Available Kani verification harnesses:$(NC)"
	@grep -r "#\[kani::proof\]" src/ -A 1 | grep "fn " | sed 's/.*fn \([^(]*\).*/  - \1/' | sort | uniq

# Check Kani configuration
kani-check:
	@echo "$(BLUE)Checking Kani configuration...$(NC)"
	@if command -v kani &> /dev/null; then \
		echo "$(GREEN)✅ Kani is installed$(NC)"; \
		kani --version; \
	else \
		echo "$(RED)❌ Kani is not installed$(NC)"; \
	fi
	@if [ -f "kani-config.toml" ]; then \
		echo "$(GREEN)✅ Kani config file exists$(NC)"; \
	else \
		echo "$(YELLOW)⚠️ Kani config file not found$(NC)"; \
	fi

# Check dependency resolution
dep-check:
	@echo "$(BLUE)Checking dependency resolution...$(NC)"
	@cargo check --workspace
	@cargo test --no-run --workspace
	@echo "$(GREEN)All dependencies resolved successfully!$(NC)"

# Continuous integration target
kani-ci: kani-setup
	@echo "$(BLUE)Running CI verification suite...$(NC)"
	@mkdir -p verification_results
	# Run only fast, deterministic verifications for CI
	cargo kani --harness verify_encryption_type_determination --timeout=60
	cargo kani --harness verify_layer_state_transitions --timeout=60
	cargo kani --harness verify_mining_stats --timeout=90
	@echo "$(GREEN)CI verification complete!$(NC)"
