# Makefile for PolyTorus ContainerLab Environment

.PHONY: help build setup test test-advanced monitor clean check

help:
	@echo "🚀 PolyTorus ContainerLab Environment"
	@echo "===================================="
	@echo ""
	@echo "Available targets:"
	@echo "  build        - Build the PolyTorus Docker image"
	@echo "  setup        - Deploy ContainerLab topology"  
	@echo "  test         - Run basic transaction tests"
	@echo "  test-advanced - Run advanced transaction tests"
	@echo "  monitor      - Start network monitoring"
	@echo "  check        - Check Rust code compilation"
	@echo "  clean        - Clean up ContainerLab environment"
	@echo ""
	@echo "Quick start:"
	@echo "  make build && make setup && make test"

build:
	@echo "🔨 Building PolyTorus Docker image..."
	docker build -t polytorus:latest .

check:
	@echo "🧪 Checking Rust compilation..."
	cargo check

setup:
	@echo "🚀 Setting up ContainerLab environment..."
	./setup_containerlab.sh

test:
	@echo "🧪 Running basic transaction tests..."
	./test_transactions.sh

test-advanced:
	@echo "🔬 Running advanced transaction tests..."
	./test_advanced.sh

monitor:
	@echo "📊 Starting network monitor..."
	./monitor_network.sh

clean:
	@echo "🧹 Cleaning up ContainerLab environment..."
	./cleanup_containerlab.sh
