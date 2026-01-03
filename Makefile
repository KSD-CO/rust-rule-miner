.PHONY: help all examples simple medium complex integration ci check fmt fmt-check clippy test build doc-test bench test-features clean

# Default target
help:
	@echo "Available targets:"
	@echo ""
	@echo "CI & Testing:"
	@echo "  make ci                     - Full CI check (matches GitHub Actions)"
	@echo "  make check                  - Quick check (fmt, clippy, test only)"
	@echo "  make fmt                    - Format code"
	@echo "  make fmt-check              - Check code formatting"
	@echo "  make clippy                 - Run clippy linter"
	@echo "  make test                   - Run tests"
	@echo "  make test-features          - Test all feature combinations"
	@echo "  make build                  - Build project"
	@echo "  make doc-test               - Run documentation tests"
	@echo "  make bench                  - Run benchmarks"
	@echo "  make clean                  - Clean build artifacts"
	@echo ""
	@echo "Examples:"
	@echo "  make all                    - Run all examples"
	@echo "  make simple                 - Run simple examples"
	@echo "  make medium                 - Run medium complexity examples"
	@echo "  make complex                - Run complex examples"
	@echo "  make integration            - Run integration examples"
	@echo "  make cloud                  - Run cloud examples (requires cloud feature)"
	@echo ""
	@echo "Individual examples:"
	@echo "  make 01_simple"
	@echo "  make 02_medium_complexity"
	@echo "  make 03_complex"
	@echo "  make integration_with_engine"
	@echo "  make excelstream_demo"
	@echo "  make cloud_demo"
	@echo "  make performance_test"

# Run all examples
all: simple medium complex integration

# Simple Examples
simple:
	@echo "=== Running Simple Examples ==="
	cargo run --example 01_simple

# Medium Complexity Examples
medium:
	@echo "=== Running Medium Complexity Examples ==="
	cargo run --example 02_medium_complexity

# Complex Examples
complex:
	@echo "=== Running Complex Examples ==="
	cargo run --example 03_complex

# Integration Examples
integration:
	@echo "=== Running Integration Examples ==="
	cargo run --example integration_with_engine
	cargo run --example excelstream_demo

# Cloud Examples (requires cloud feature)
cloud:
	@echo "=== Running Cloud Examples ==="
	cargo run --example cloud_demo --features cloud

# Performance Test
performance:
	@echo "=== Running Performance Test ==="
	cargo run --example performance_test --release

# CI checks (exactly matching GitHub Actions workflow)
ci: fmt-check clippy build test test-features doc-test bench-check
	@echo "✅ All CI checks passed!"

# Test different feature combinations (critical for catching feature-specific bugs)
test-features:
	@echo "🧪 Testing feature combinations..."
	@echo "  Testing: no features (default)"
	@cargo test --no-default-features --lib
	@echo "  Testing: cloud only"
	@cargo test --no-default-features --features cloud --lib
	@echo "  Testing: all features"
	@cargo test --all-features --lib
	@echo "✅ All feature combinations passed!"

# Check everything (quick check without build)
check: fmt clippy test
	@echo "✅ All checks passed!"

# Format code
fmt:
	@echo "🔧 Formatting code..."
	@cargo fmt

# Check formatting
fmt-check:
	@echo "🔍 Checking code formatting..."
	@cargo fmt -- --check

# Run clippy
clippy:
	@echo "🔍 Running clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
	@echo "🧪 Running tests..."
	@cargo test --verbose --all-features

# Build project
build:
	@echo "🔨 Building project..."
	@cargo build --verbose --all-features

# Run doc tests
doc-test:
	@echo "📚 Running doc tests..."
	@cargo test --doc --verbose

# Run benchmarks
bench:
	@echo "⚡ Running benchmarks..."
	@cargo bench --verbose

# Check benchmarks compile (for CI)
bench-check:
	@echo "⚡ Checking benchmarks compile..."
	@cargo bench --no-run --verbose

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean

# Individual example targets
01_simple:
	cargo run --example 01_simple

02_medium_complexity:
	cargo run --example 02_medium_complexity

03_complex:
	cargo run --example 03_complex

integration_with_engine:
	cargo run --example integration_with_engine

excelstream_demo:
	cargo run --example excelstream_demo

cloud_demo:
	cargo run --example cloud_demo --features cloud

performance_test:
	cargo run --example performance_test --release
