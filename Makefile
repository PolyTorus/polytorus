# Makefile for Polytorus Kani Verification

.PHONY: kani-install kani-setup kani-verify kani-clean kani-quick kani-crypto kani-blockchain kani-modular help

# Colors for output
BLUE := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[1;33m
RED := \033[0;31m
NC := \033[0m # No Color

# Default target
help:
	@echo "$(BLUE)Polytorus Kani Verification Makefile$(NC)"
	@echo ""
	@echo "Available targets:"
	@echo "  $(GREEN)kani-install$(NC)     - Install Kani verifier"
	@echo "  $(GREEN)kani-setup$(NC)       - Setup Kani for this project"
	@echo "  $(GREEN)kani-verify$(NC)      - Run all Kani verifications"
	@echo "  $(GREEN)kani-quick$(NC)       - Run quick verification subset"
	@echo "  $(GREEN)kani-crypto$(NC)      - Run cryptographic verifications only"
	@echo "  $(GREEN)kani-blockchain$(NC)  - Run blockchain verifications only"
	@echo "  $(GREEN)kani-modular$(NC)     - Run modular architecture verifications only"
	@echo "  $(GREEN)kani-clean$(NC)       - Clean verification results"
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
	./scripts/run_kani_verification.sh

# Run quick verification (subset for development)
kani-quick: kani-setup
	@echo "$(BLUE)Running quick Kani verification...$(NC)"
	@mkdir -p verification_results
	cargo kani --harness verify_encryption_type_determination
	cargo kani --harness verify_mining_stats
	cargo kani --harness verify_layer_state_transitions
	@echo "$(GREEN)Quick verification complete!$(NC)"

# Run cryptographic verifications only
kani-crypto: kani-setup
	@echo "$(BLUE)Running cryptographic verifications...$(NC)"
	@mkdir -p verification_results
	-cargo kani --harness verify_ecdsa_sign_verify 2>&1 | tee verification_results/ecdsa.log
	-cargo kani --harness verify_encryption_type_determination 2>&1 | tee verification_results/encryption_type.log
	-cargo kani --harness verify_transaction_integrity 2>&1 | tee verification_results/transaction_integrity.log
	-cargo kani --harness verify_transaction_value_bounds 2>&1 | tee verification_results/transaction_bounds.log
	@echo "$(GREEN)Cryptographic verifications complete!$(NC)"

# Run blockchain verifications only
kani-blockchain: kani-setup
	@echo "$(BLUE)Running blockchain verifications...$(NC)"
	@mkdir -p verification_results
	-cargo kani --harness verify_mining_stats 2>&1 | tee verification_results/mining_stats.log
	-cargo kani --harness verify_mining_attempts 2>&1 | tee verification_results/mining_attempts.log
	-cargo kani --harness verify_difficulty_adjustment_config 2>&1 | tee verification_results/difficulty_config.log
	-cargo kani --harness verify_difficulty_bounds 2>&1 | tee verification_results/difficulty_bounds.log
	-cargo kani --harness verify_block_hash_consistency 2>&1 | tee verification_results/block_hash.log
	@echo "$(GREEN)Blockchain verifications complete!$(NC)"

# Run modular architecture verifications only
kani-modular: kani-setup
	@echo "$(BLUE)Running modular architecture verifications...$(NC)"
	@mkdir -p verification_results
	-cargo kani --harness verify_message_priority_ordering 2>&1 | tee verification_results/message_priority.log
	-cargo kani --harness verify_layer_state_transitions 2>&1 | tee verification_results/layer_states.log
	-cargo kani --harness verify_message_bus_capacity 2>&1 | tee verification_results/message_bus.log
	-cargo kani --harness verify_orchestrator_coordination 2>&1 | tee verification_results/orchestrator.log
	-cargo kani --harness verify_data_availability_properties 2>&1 | tee verification_results/data_availability.log
	@echo "$(GREEN)Modular architecture verifications complete!$(NC)"

# Clean verification results
kani-clean:
	@echo "$(YELLOW)Cleaning verification results...$(NC)"
	rm -rf verification_results
	@echo "$(GREEN)Clean complete!$(NC)"

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

# Continuous integration target
kani-ci: kani-setup
	@echo "$(BLUE)Running CI verification suite...$(NC)"
	@mkdir -p verification_results
	# Run only fast, deterministic verifications for CI
	cargo kani --harness verify_encryption_type_determination --timeout=60
	cargo kani --harness verify_layer_state_transitions --timeout=60
	cargo kani --harness verify_mining_stats --timeout=90
	@echo "$(GREEN)CI verification complete!$(NC)"
