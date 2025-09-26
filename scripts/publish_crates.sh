#!/bin/bash
set -euo pipefail

# Crates.io publishing script
# Publishes workspace crates in dependency order

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=== Centotype Crates.io Publishing ==="

# Check if we're on a release tag
if [[ "${GITHUB_REF:-}" =~ ^refs/tags/v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "✅ Publishing from release tag: ${GITHUB_REF}"
else
    echo "⚠️  Not on a release tag, proceeding anyway"
fi

# Ensure we're logged in to crates.io
if ! cargo login --help &>/dev/null; then
    echo "❌ Cargo not available"
    exit 1
fi

# Publication order (dependencies first)
CRATES=(
    "platform"
    "core"
    "content"
    "persistence"
    "analytics"
    "engine"
    "cli"
    "centotype-bin"
)

# Dry run first
echo
echo "🔍 Performing dry run..."
for crate in "${CRATES[@]}"; do
    echo "Checking $crate..."
    cd "$PROJECT_ROOT/$crate"

    # Verify package can be built
    cargo check --release

    # Dry run publish
    cargo publish --dry-run

    echo "✅ $crate ready for publishing"
done

# Confirm publication
echo
read -p "Proceed with actual publication? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Publication cancelled"
    exit 0
fi

# Actual publication
echo
echo "🚀 Publishing crates..."

for crate in "${CRATES[@]}"; do
    echo
    echo "Publishing $crate..."
    cd "$PROJECT_ROOT/$crate"

    # Publish with retry logic
    max_attempts=3
    attempt=1

    while [[ $attempt -le $max_attempts ]]; do
        echo "Attempt $attempt/$max_attempts for $crate"

        if cargo publish; then
            echo "✅ $crate published successfully"
            break
        else
            if [[ $attempt -eq $max_attempts ]]; then
                echo "❌ Failed to publish $crate after $max_attempts attempts"
                exit 1
            fi

            echo "⚠️  Attempt $attempt failed, retrying in 30 seconds..."
            sleep 30
            ((attempt++))
        fi
    done

    # Wait between publications to avoid rate limiting
    if [[ "$crate" != "${CRATES[-1]}" ]]; then
        echo "Waiting 10 seconds before next publication..."
        sleep 10
    fi
done

echo
echo "🎉 All crates published successfully!"
echo
echo "Verify publications at:"
for crate in "${CRATES[@]}"; do
    # Convert directory name to crate name
    if [[ "$crate" == "centotype-bin" ]]; then
        crate_name="centotype"
    else
        crate_name="centotype-$crate"
    fi
    echo "  https://crates.io/crates/$crate_name"
done