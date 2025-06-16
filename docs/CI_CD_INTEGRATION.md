# PolyTorus CI/CD Integration Guide

## Overview
PolyTorus features a comprehensive CI/CD pipeline designed for modern software development practices, including automated testing, security scanning, and deployment workflows.

## 🚀 Features (June 2025)

### Automated Pre-commit Hooks
- **Code Formatting**: Automatic `cargo fmt` execution
- **Linting**: Comprehensive `cargo clippy` checks
- **Quick Testing**: Fast test suite execution
- **Commit Prevention**: Prevents commits that don't meet quality standards

### GitHub Actions Pipeline
- **Multi-platform Testing**: Linux, macOS, and Windows support
- **Code Coverage**: Comprehensive test coverage reporting
- **Security Scanning**: Automated vulnerability detection
- **Formal Verification**: Kani proof verification
- **Docker Integration**: Automated container builds and scanning

### Quality Enforcement
- **Zero Warning Policy**: No warnings allowed in codebase
- **Automated Formatting**: Consistent code style enforcement
- **Security Auditing**: Regular dependency vulnerability scanning
- **Documentation Coverage**: All public APIs must be documented

## Pre-commit Hook Setup

### Automatic Installation
The pre-commit hook is automatically installed when you run:

```bash
make pre-commit
```

### Manual Installation
If you need to manually install the pre-commit hook:

```bash
# Make the hook executable
chmod +x .git/hooks/pre-commit

# Test the hook
.git/hooks/pre-commit
```

### What the Pre-commit Hook Does

1. **Format Check**: Runs `cargo fmt --all --check`
2. **Lint Check**: Runs `cargo clippy --all-targets --all-features -- -D warnings`
3. **Quick Tests**: Runs `cargo test --lib` for fast feedback
4. **File Analysis**: Only checks modified Rust files for efficiency

### Pre-commit Hook Output Example
```bash
🔍 Running pre-commit checks...
📝 Checking Rust files: src/main.rs src/lib.rs src/crypto/mod.rs
🔧 Checking code formatting...
📎 Running clippy...
🧪 Running quick tests...
✅ All pre-commit checks passed!
```

## GitHub Actions Workflow

### Workflow Overview
The unified CI/CD pipeline (`.github/workflows/main.yml`) includes:

```yaml
jobs:
  quick-checks:      # Fast feedback (formatting, linting, security)
  test:             # Multi-platform comprehensive testing
  coverage:         # Code coverage with codecov integration
  kani-verification: # Formal verification of critical components
  docker:           # Container builds with security scanning
  security:         # Comprehensive security auditing
  deploy:           # Production deployment (on version tags)
```

### Job Details

#### Quick Checks Job
- **Purpose**: Provide fast feedback on basic quality issues
- **Runtime**: ~2-3 minutes
- **Checks**: 
  - Code formatting (`cargo fmt --check`)
  - Linting (`cargo clippy`)
  - Security audit (`cargo audit`)

#### Test Job
- **Platforms**: Ubuntu, macOS, Windows
- **Rust Versions**: Stable, beta, nightly (minimum 1.70+)
- **Test Types**: Unit tests, integration tests, documentation tests
- **Features**: Tests with all features enabled

#### Coverage Job
- **Tool**: cargo-tarpaulin
- **Output**: Codecov integration
- **Threshold**: Maintains >80% coverage
- **Reporting**: Detailed coverage reports in PRs

#### Kani Verification Job
- **Purpose**: Formal verification of critical cryptographic functions
- **Components**: ECDSA, transaction validation, consensus logic
- **Safety**: Memory safety proofs, overflow checking

#### Docker Job
- **Multi-stage**: Optimized build process
- **Security**: Vulnerability scanning with Trivy
- **Platforms**: AMD64, ARM64
- **Registry**: GitHub Container Registry (ghcr.io)

#### Security Job
- **Tools**: cargo-audit, cargo-deny, dependency scanning
- **Checks**: Known vulnerabilities, license compliance
- **Integration**: Dependabot for automated updates

### Workflow Triggers

```yaml
on:
  push:
    branches: [ main, develop ]
    tags: [ 'v*' ]
  pull_request:
    branches: [ main, develop ]
```

## Development Workflow

### Local Development
1. **Make Changes**: Edit code normally
2. **Pre-commit Check**: Automatic check on `git commit`
3. **Fix Issues**: Address any formatting or linting issues
4. **Commit**: Successful commit after all checks pass

### Recommended Development Commands
```bash
# Before starting work
make pre-commit          # Ensure environment is ready

# During development
make fmt                 # Format code
make clippy             # Check linting
make test               # Run tests

# Before committing
make ci-verify-quick    # Quick CI simulation
make ci-verify          # Full CI simulation (slower)

# Git workflow
git add .
git commit -m "Your message"  # Pre-commit hook runs automatically
git push
```

### Pull Request Workflow
1. **Create PR**: All checks run automatically
2. **Review Results**: Check CI status in PR
3. **Fix Issues**: Address any CI failures
4. **Merge**: Automatic deployment on approved PRs to main

## Docker Integration

### Development Environment
```bash
# Quick development setup
docker-compose -f docker-compose.dev.yml up

# With custom environment
cp .env.example .env
# Edit .env as needed
docker-compose -f docker-compose.dev.yml up
```

### Production Environment
```bash
# Production deployment
cp .env.secrets.example .env.secrets
# Edit .env.secrets with production values
docker-compose -f docker-compose.prod.yml up -d
```

### Container Features
- **Multi-stage Build**: Optimized image size
- **Security**: Non-root user, minimal base image
- **Health Checks**: Built-in container health monitoring
- **Secrets**: Docker secrets integration for sensitive data

## Security Integration

### Automated Security Scanning
- **Dependency Scanning**: cargo-audit on every commit
- **License Compliance**: cargo-deny for license checking
- **Container Scanning**: Trivy security scanner for Docker images
- **Dependabot**: Automated dependency updates

### Security Policies
- **Zero Vulnerabilities**: No known vulnerabilities allowed
- **License Compliance**: Only approved licenses (MIT, Apache-2.0)
- **Regular Updates**: Weekly automated dependency updates
- **Security Advisories**: Immediate notifications on new vulnerabilities

## Monitoring and Observability

### CI/CD Metrics
- **Build Times**: Tracked across all platforms and jobs
- **Success Rates**: Monitor build success/failure rates
- **Coverage Trends**: Track code coverage over time
- **Security Issues**: Alert on new vulnerabilities

### Performance Monitoring
- **Test Performance**: Track test suite execution time
- **Build Performance**: Monitor compilation and build times
- **Resource Usage**: Memory and CPU usage during CI

## Troubleshooting

### Common Issues

#### Pre-commit Hook Failures
```bash
# Format issues
cargo fmt --all

# Clippy warnings
cargo clippy --all-targets --all-features --fix

# Test failures
cargo test --lib
```

#### CI/CD Pipeline Issues
```bash
# Simulate CI locally
make ci-verify

# Check specific components
make fmt clippy test audit

# Docker issues
docker-compose -f docker-compose.dev.yml build
```

#### Environment Issues
```bash
# Reset development environment
make clean
cargo clean
docker-compose down --volumes

# Rebuild everything
make build
docker-compose -f docker-compose.dev.yml up --build
```

### Getting Help
- **CI Logs**: Check GitHub Actions logs for detailed error information
- **Local Simulation**: Use `make ci-verify` to reproduce CI issues locally
- **Docker Logs**: Use `docker-compose logs` for container issues
- **Documentation**: Check individual component documentation in `docs/`

## Best Practices

### Code Quality
1. **Run pre-commit checks** before pushing
2. **Keep commits small** and focused
3. **Write descriptive commit messages**
4. **Add tests** for new functionality
5. **Update documentation** as needed

### Security
1. **Never commit secrets** to version control
2. **Use environment variables** for configuration
3. **Keep dependencies updated** via Dependabot
4. **Review security advisories** regularly

### Performance
1. **Profile CI changes** to avoid slowdowns
2. **Use caching** effectively (Rust cache, Docker cache)
3. **Minimize test data** in CI environment
4. **Optimize Docker layers** for faster builds

## Future Enhancements

### Planned Features
- **Parallel Testing**: Further parallelization of test suite
- **Advanced Metrics**: More detailed CI/CD analytics
- **Deployment Automation**: Zero-downtime production deployments
- **Environment Promotion**: Automated staging to production promotion
- **Integration Testing**: Cross-service integration testing
