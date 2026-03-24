default:
    @just --list

# Initial setup: configure git hooks
setup:
    git config core.hooksPath .githooks

# Format all code
fmt:
    cargo fmt

# Check formatting
fmt-check:
    cargo fmt --check

# Run clippy
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
    cargo test

# Build in release mode
build:
    cargo build --release

# Run all checks (format, lint, test)
check: fmt-check lint test

# Create a release tag, push it, and create a GitHub Release (usage: just release 0.5.0)
release version:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo_version=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
    if [ "$cargo_version" != "{{version}}" ]; then
        echo "Error: Cargo.toml version ($cargo_version) does not match {{version}}"
        exit 1
    fi
    if [ -n "$(git status --porcelain)" ]; then
        echo "Error: working directory is not clean"
        exit 1
    fi
    branch=$(git branch --show-current)
    if [ "$branch" != "main" ]; then
        echo "Error: not on main branch (on $branch)"
        exit 1
    fi
    # Extract release notes from CHANGELOG.md for this version
    notes=$(awk '/^## \['"{{version}}"'\]/{found=1; next} /^## \[/{if(found) exit} found{print}' CHANGELOG.md)
    if [ -z "$notes" ]; then
        echo "Error: no changelog entry found for [{{version}}] in CHANGELOG.md"
        exit 1
    fi
    # Verify [Unreleased] section exists (for next development cycle)
    if ! grep -q '^\## \[Unreleased\]' CHANGELOG.md; then
        echo "Error: CHANGELOG.md is missing [Unreleased] section"
        exit 1
    fi
    git tag -a "v{{version}}" -m "v{{version}}"
    git push origin "v{{version}}"
    echo "$notes" | gh release create "v{{version}}" --title "v{{version}}" --notes-file -
    echo "Tagged, pushed, and released v{{version}}"
