#!/bin/bash
set -euo pipefail

# Release preparation script
# Prepares cross-platform release assets from build artifacts

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ARTIFACTS_DIR="$PROJECT_ROOT/artifacts"
RELEASE_DIR="$PROJECT_ROOT/release-assets"

echo "=== Centotype Release Asset Preparation ==="
echo "Artifacts directory: $ARTIFACTS_DIR"
echo "Release directory: $RELEASE_DIR"

# Clean and create release directory
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

# Extract version from Cargo.toml
VERSION=$(grep '^version =' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/.*= "//' | sed 's/".*//')
echo "Version: $VERSION"

# Platform configurations
declare -A PLATFORMS=(
    ["x86_64-unknown-linux-gnu"]="linux-x64"
    ["aarch64-unknown-linux-gnu"]="linux-arm64"
    ["x86_64-apple-darwin"]="macos-x64"
    ["aarch64-apple-darwin"]="macos-arm64"
    ["x86_64-pc-windows-msvc"]="windows-x64"
    ["aarch64-pc-windows-msvc"]="windows-arm64"
)

# Process each platform
for rust_target in "${!PLATFORMS[@]}"; do
    platform_name="${PLATFORMS[$rust_target]}"
    artifact_dir="$ARTIFACTS_DIR/centotype-$rust_target"

    echo
    echo "Processing platform: $platform_name ($rust_target)"

    if [[ ! -d "$artifact_dir" ]]; then
        echo "⚠️  Skipping $platform_name - artifact directory not found"
        continue
    fi

    # Find the binary
    if [[ "$rust_target" == *"windows"* ]]; then
        binary_name="centotype.exe"
    else
        binary_name="centotype"
    fi

    binary_path="$artifact_dir/$binary_name"

    if [[ ! -f "$binary_path" ]]; then
        echo "⚠️  Skipping $platform_name - binary not found: $binary_path"
        continue
    fi

    echo "✅ Found binary: $binary_path"

    # Create platform-specific release package
    package_name="centotype-v$VERSION-$platform_name"
    package_dir="$RELEASE_DIR/$package_name"
    mkdir -p "$package_dir"

    # Copy binary
    cp "$binary_path" "$package_dir/"

    # Copy documentation and license files
    cp "$PROJECT_ROOT/README.md" "$package_dir/"
    cp "$PROJECT_ROOT/LICENSE" "$package_dir/" 2>/dev/null || echo "License file not found"

    # Create installation instructions
    cat > "$package_dir/INSTALL.md" << EOF
# Centotype v$VERSION Installation

## Quick Install

### Option 1: Run directly
\`\`\`bash
./$binary_name --help
\`\`\`

### Option 2: Install to system PATH

#### Linux/macOS:
\`\`\`bash
sudo cp $binary_name /usr/local/bin/
centotype --help
\`\`\`

#### Windows:
Copy \`$binary_name\` to a directory in your PATH environment variable.

## Usage

Run \`centotype --help\` for available commands and options.

## System Requirements

- Terminal with UTF-8 support
- Minimum terminal size: 80x24 characters

## Troubleshooting

If you encounter issues:
1. Ensure your terminal supports ANSI escape sequences
2. Try running with \`--debug\` flag for verbose output
3. Check the GitHub repository for known issues

Repository: https://github.com/centotype/centotype
EOF

    # Create archive
    if [[ "$rust_target" == *"windows"* ]]; then
        # Use zip for Windows
        (cd "$RELEASE_DIR" && zip -r "$package_name.zip" "$package_name/")
        echo "📦 Created: $package_name.zip"
    else
        # Use tar.gz for Unix-like systems
        (cd "$RELEASE_DIR" && tar -czf "$package_name.tar.gz" "$package_name/")
        echo "📦 Created: $package_name.tar.gz"
    fi

    # Also create standalone binary with platform suffix
    standalone_name="centotype-v$VERSION-$platform_name"
    if [[ "$rust_target" == *"windows"* ]]; then
        standalone_name="$standalone_name.exe"
    fi
    cp "$binary_path" "$RELEASE_DIR/$standalone_name"
    echo "📦 Created standalone: $standalone_name"

    # Clean up temporary directory
    rm -rf "$package_dir"
done

# Create checksums
echo
echo "Generating checksums..."
cd "$RELEASE_DIR"

# SHA256 checksums
sha256sum * > "centotype-v$VERSION-checksums.sha256" 2>/dev/null || \
shasum -a 256 * > "centotype-v$VERSION-checksums.sha256"

echo "✅ Checksums generated"

# Create release notes template
cat > "$RELEASE_DIR/release-notes.md" << EOF
# Centotype v$VERSION

## Installation

Download the appropriate binary for your platform:

$(for file in *.tar.gz *.zip; do
    [[ -f "$file" ]] && echo "- **${file}**"
done)

## Verify Download

\`\`\`bash
# Download checksums
curl -LO https://github.com/centotype/centotype/releases/download/v$VERSION/centotype-v$VERSION-checksums.sha256

# Verify your download
sha256sum -c centotype-v$VERSION-checksums.sha256
\`\`\`

## Quick Start

\`\`\`bash
# Extract (Linux/macOS)
tar -xzf centotype-v$VERSION-*.tar.gz
cd centotype-v$VERSION-*/

# Run
./centotype --help
\`\`\`

## Changes

<!-- Add release notes here -->

## Technical Notes

- Built with Rust $(rustc --version 2>/dev/null || echo "unknown")
- Performance targets validated in CI
- Cross-platform compatibility tested

EOF

echo "✅ Release notes template created"

# Summary
echo
echo "=== Release Preparation Complete ==="
echo "Release assets directory: $RELEASE_DIR"
echo "Generated files:"
ls -la "$RELEASE_DIR"

echo
echo "Next steps:"
echo "1. Review release notes in release-notes.md"
echo "2. Test installation on target platforms"
echo "3. Upload to GitHub releases"
echo "4. Update package registries (cargo, npm)"