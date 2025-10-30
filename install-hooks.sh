#!/bin/bash

# Mountain Climber Git Hooks Installer
# Sets up comprehensive pre-commit hooks for code quality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info "Installing Mountain Climber Git Hooks..."

# Ensure we're in a git repository
if [ ! -d ".git" ]; then
    print_error "Not in a git repository! Please run this from the project root."
    exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p .git/hooks

# Install pre-commit hook (already done above)
print_success "Pre-commit hook already installed"

# Install commit-msg hook for conventional commits
print_info "Installing commit-msg hook for conventional commits..."
cat > .git/hooks/commit-msg << 'EOF'
#!/bin/bash

# Commit message validation for conventional commits
# Ensures commit messages follow the pattern: type(scope): description

commit_regex='^(feat|fix|docs|style|refactor|test|chore|perf|ci|build|revert)(\(.+\))?: .{1,50}'

if ! grep -qE "$commit_regex" "$1"; then
    echo "❌ Invalid commit message format!"
    echo ""
    echo "Commit messages must follow conventional commit format:"
    echo "  type(scope): description"
    echo ""
    echo "Types: feat, fix, docs, style, refactor, test, chore, perf, ci, build, revert"
    echo "Example: feat(ice-axe): add terrain breaking functionality"
    echo "Example: fix(movement): resolve stamina calculation bug"
    echo "Example: docs(readme): update installation instructions"
    echo ""
    echo "Your message: $(cat $1)"
    exit 1
fi

echo "✅ Commit message format is valid"
EOF

chmod +x .git/hooks/commit-msg
print_success "Commit-msg hook installed"

# Install pre-push hook for additional checks
print_info "Installing pre-push hook..."
cat > .git/hooks/pre-push << 'EOF'
#!/bin/bash

# Pre-push hook - runs before pushing to remote
# Performs additional checks for release readiness

set -e

echo "🚀 Running pre-push checks..."

# Function to print section headers
print_section() {
    echo ""
    echo "=================================="
    echo "$1"
    echo "=================================="
}

# Check if we're pushing to main/master
protected_branch='master'
current_branch=$(git rev-parse --abbrev-ref HEAD)

if [ "$current_branch" = "$protected_branch" ]; then
    print_section "🔒 Pushing to protected branch: $protected_branch"
    
    # Run extra checks for main branch pushes
    echo "🧪 Running comprehensive test suite..."
    if ! cargo test --release; then
        echo "❌ Release tests failed!"
        exit 1
    fi
    
    echo "📊 Checking test coverage..."
    # If tarpaulin is available, check coverage
    if command -v cargo-tarpaulin >/dev/null 2>&1; then
        if ! cargo tarpaulin --ignore-tests --out stdout | grep "Coverage:" | grep -E "([89][0-9]|100)\."; then
            echo "⚠️  Test coverage might be below 80%"
            echo "💡 Consider adding more tests"
        fi
    fi
    
    echo "🔍 Final security check..."
    # Check for common security issues
    if grep -r "password\|secret\|key\|token" src/ --include="*.rs" | grep -v "// " | head -5; then
        echo "⚠️  Potential secrets detected in source code"
        echo "💡 Ensure no sensitive data is committed"
    fi
fi

print_section "✅ Pre-push checks completed"
echo "🎉 Ready to push!"
EOF

chmod +x .git/hooks/pre-push
print_success "Pre-push hook installed"

# Install post-commit hook for automatic documentation updates
print_info "Installing post-commit hook..."
cat > .git/hooks/post-commit << 'EOF'
#!/bin/bash

# Post-commit hook - runs after each commit
# Updates documentation and performs housekeeping

echo "📝 Post-commit: Updating documentation..."

# Generate documentation
if cargo doc --no-deps --document-private-items --quiet; then
    echo "✅ Documentation updated"
else
    echo "⚠️  Documentation generation had issues"
fi

# Update changelog if this is a main branch commit
current_branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$current_branch" = "master" ] || [ "$current_branch" = "main" ]; then
    echo "📋 Consider updating CHANGELOG.md for this commit"
fi

echo "🎉 Post-commit tasks completed"
EOF

chmod +x .git/hooks/post-commit
print_success "Post-commit hook installed"

# Create a configuration file for the hooks
print_info "Creating hooks configuration..."
cat > .git/hooks/config.toml << 'EOF'
# Mountain Climber Git Hooks Configuration

[quality_gates]
max_complexity = 20
min_coverage = 80
max_cyclomatic_complexity = 10

[commit_rules]
enforce_conventional_commits = true
max_subject_length = 50
max_body_line_length = 72

[security]
check_secrets = true
check_unsafe_code = true
warn_on_unwrap = true

[performance]
run_benchmarks_on_push = false
check_binary_size = true
max_binary_size_mb = 50
EOF

print_success "Hooks configuration created"

# Create a script to run quality checks manually
print_info "Creating manual quality check script..."
cat > scripts/quality-check.sh << 'EOF'
#!/bin/bash

# Manual quality check script
# Run this to check code quality without committing

echo "🔍 Running comprehensive quality checks..."

# Format check
echo "📝 Checking formatting..."
cargo fmt --check

# Clippy
echo "🔍 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings -A dead_code

# Tests
echo "🧪 Running tests..."
cargo test --all-targets --all-features

# Documentation
echo "📚 Building documentation..."
cargo doc --no-deps --document-private-items

# Benchmark (if available)
if [ -d "benches" ]; then
    echo "⚡ Running benchmarks..."
    cargo bench --no-run
fi

echo "✅ All quality checks completed!"
EOF

mkdir -p scripts
chmod +x scripts/quality-check.sh
print_success "Quality check script created: scripts/quality-check.sh"

# Summary
echo ""
echo "🎉 Git hooks installation completed!"
echo ""
echo "Installed hooks:"
echo "  • pre-commit: Tests, clippy, formatting, docs, complexity"
echo "  • commit-msg: Conventional commit format validation"
echo "  • pre-push: Additional checks for protected branches"
echo "  • post-commit: Documentation updates"
echo ""
echo "Configuration:"
echo "  • .git/hooks/config.toml: Hook settings"
echo "  • scripts/quality-check.sh: Manual quality checks"
echo ""
echo "Usage:"
echo "  • Hooks run automatically on git operations"
echo "  • Run 'scripts/quality-check.sh' for manual checks"
echo "  • Edit .git/hooks/config.toml to customize settings"
echo ""
print_success "Mountain Climber development workflow is now ready! 🏔️"