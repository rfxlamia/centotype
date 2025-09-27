#!/bin/bash
set -euo pipefail

# NPM package preparation script
# Creates npm wrapper package for cross-platform binary distribution

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NPM_DIR="$PROJECT_ROOT/npm-package"

# Extract version from Cargo.toml
VERSION=$(grep '^version =' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/.*= "//' | sed 's/".*//')

echo "=== Centotype NPM Package Preparation ==="
echo "Version: $VERSION"
echo "NPM directory: $NPM_DIR"

# Clean and create npm package directory
rm -rf "$NPM_DIR"
mkdir -p "$NPM_DIR/bin"

# Create package.json
cat > "$NPM_DIR/package.json" << EOF
{
  "name": "centotype",
  "version": "$VERSION",
  "description": "CLI-based typing trainer with 100 progressive difficulty levels",
  "main": "index.js",
  "bin": {
    "centotype": "./bin/centotype"
  },
  "scripts": {
    "postinstall": "node install.js",
    "test": "node test.js"
  },
  "keywords": [
    "typing",
    "cli",
    "trainer",
    "practice",
    "wpm",
    "terminal",
    "speed",
    "accuracy"
  ],
  "author": "Centotype Team <contact@centotype.dev>",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/rfxlamia/centotype.git"
  },
  "homepage": "https://centotype.dev",
  "bugs": {
    "url": "https://github.com/rfxlamia/centotype/issues"
  },
  "engines": {
    "node": ">=12.0.0"
  },
  "os": [
    "darwin",
    "linux",
    "win32"
  ],
  "cpu": [
    "x64",
    "arm64"
  ],
  "files": [
    "bin/",
    "install.js",
    "test.js",
    "index.js",
    "README.md"
  ]
}
EOF

# Create installation script
cat > "$NPM_DIR/install.js" << 'EOF'
#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const REPO = 'centotype/centotype';
const BINARY_NAME = process.platform === 'win32' ? 'centotype.exe' : 'centotype';

// Platform mapping
const PLATFORM_MAP = {
  'darwin-x64': 'macos-x64',
  'darwin-arm64': 'macos-arm64',
  'linux-x64': 'linux-x64',
  'linux-arm64': 'linux-arm64',
  'win32-x64': 'windows-x64',
  'win32-arm64': 'windows-arm64'
};

function getPlatform() {
  const platform = process.platform;
  const arch = process.arch === 'x64' ? 'x64' : 'arm64';
  return `${platform}-${arch}`;
}

function getDownloadUrl(version, platform) {
  const platformName = PLATFORM_MAP[platform];
  if (!platformName) {
    throw new Error(`Unsupported platform: ${platform}`);
  }

  const filename = `centotype-v${version}-${platformName}${process.platform === 'win32' ? '.exe' : ''}`;
  return `https://github.com/${REPO}/releases/download/v${version}/${filename}`;
}

function download(url, destination) {
  return new Promise((resolve, reject) => {
    console.log(`Downloading: ${url}`);

    const file = fs.createWriteStream(destination);

    https.get(url, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        // Follow redirect
        file.close();
        fs.unlinkSync(destination);
        return download(response.headers.location, destination).then(resolve).catch(reject);
      }

      if (response.statusCode !== 200) {
        file.close();
        fs.unlinkSync(destination);
        return reject(new Error(`Download failed with status: ${response.statusCode}`));
      }

      response.pipe(file);

      file.on('finish', () => {
        file.close();
        resolve();
      });

      file.on('error', (err) => {
        file.close();
        fs.unlinkSync(destination);
        reject(err);
      });
    }).on('error', (err) => {
      file.close();
      fs.unlinkSync(destination);
      reject(err);
    });
  });
}

async function install() {
  try {
    const packageJson = JSON.parse(fs.readFileSync(path.join(__dirname, 'package.json'), 'utf8'));
    const version = packageJson.version;
    const platform = getPlatform();

    console.log(`Installing Centotype v${version} for ${platform}...`);

    const binDir = path.join(__dirname, 'bin');
    if (!fs.existsSync(binDir)) {
      fs.mkdirSync(binDir, { recursive: true });
    }

    const binaryPath = path.join(binDir, BINARY_NAME);
    const downloadUrl = getDownloadUrl(version, platform);

    await download(downloadUrl, binaryPath);

    // Make executable on Unix-like systems
    if (process.platform !== 'win32') {
      fs.chmodSync(binaryPath, 0o755);
    }

    console.log('✅ Centotype installed successfully!');
    console.log('Run "npx centotype --help" to get started.');

  } catch (error) {
    console.error('❌ Installation failed:', error.message);
    console.error('');
    console.error('Alternative installation methods:');
    console.error('1. Download directly from: https://github.com/rfxlamia/centotype/releases');
    console.error('2. Install via Cargo: cargo install centotype');
    process.exit(1);
  }
}

if (require.main === module) {
  install();
}

module.exports = { install };
EOF

# Create test script
cat > "$NPM_DIR/test.js" << 'EOF'
#!/usr/bin/env node

const { execSync } = require('child_process');
const path = require('path');

const BINARY_NAME = process.platform === 'win32' ? 'centotype.exe' : 'centotype';

function test() {
  try {
    const binaryPath = path.join(__dirname, 'bin', BINARY_NAME);

    console.log('Testing Centotype installation...');

    // Test version command
    const version = execSync(`"${binaryPath}" --version`, { encoding: 'utf8' });
    console.log('✅ Version check passed:', version.trim());

    // Test help command
    const help = execSync(`"${binaryPath}" --help`, { encoding: 'utf8' });
    if (help.includes('Centotype')) {
      console.log('✅ Help command passed');
    } else {
      throw new Error('Help output does not contain expected content');
    }

    console.log('✅ All tests passed!');

  } catch (error) {
    console.error('❌ Test failed:', error.message);
    process.exit(1);
  }
}

if (require.main === module) {
  test();
}

module.exports = { test };
EOF

# Create main index.js
cat > "$NPM_DIR/index.js" << 'EOF'
const { execSync } = require('child_process');
const path = require('path');

const BINARY_NAME = process.platform === 'win32' ? 'centotype.exe' : 'centotype';

function runCentotype(args = []) {
  const binaryPath = path.join(__dirname, 'bin', BINARY_NAME);
  const command = `"${binaryPath}" ${args.join(' ')}`;

  try {
    return execSync(command, {
      encoding: 'utf8',
      stdio: 'inherit'
    });
  } catch (error) {
    throw new Error(`Failed to run Centotype: ${error.message}`);
  }
}

module.exports = {
  run: runCentotype,
  binaryPath: path.join(__dirname, 'bin', BINARY_NAME)
};
EOF

# Create README for npm
cat > "$NPM_DIR/README.md" << EOF
# Centotype

A CLI-based typing trainer with 100 progressive difficulty levels.

## Installation

\`\`\`bash
npm install -g centotype
\`\`\`

## Usage

\`\`\`bash
# Start training
centotype play --level 1

# Practice specific patterns
centotype drill --category symbols

# Endurance mode
centotype endurance --duration 15

# View progress
centotype stats
\`\`\`

## Features

- 100 progressive difficulty levels
- Real-time performance metrics
- Customizable training modes
- Cross-platform compatibility
- Terminal-based interface

## System Requirements

- Node.js 12.0.0 or higher
- Terminal with UTF-8 support
- Minimum terminal size: 80x24 characters

## Programmatic Usage

\`\`\`javascript
const centotype = require('centotype');

// Run with arguments
centotype.run(['--help']);

// Get binary path
console.log(centotype.binaryPath);
\`\`\`

## Links

- [Homepage](https://centotype.dev)
- [GitHub Repository](https://github.com/rfxlamia/centotype)
- [Issue Tracker](https://github.com/rfxlamia/centotype/issues)

## License

MIT - see LICENSE file for details.
EOF

# Make scripts executable
chmod +x "$NPM_DIR/install.js"
chmod +x "$NPM_DIR/test.js"

echo "✅ NPM package prepared at: $NPM_DIR"
echo
echo "Files created:"
ls -la "$NPM_DIR"

echo
echo "To publish:"
echo "1. cd $NPM_DIR"
echo "2. npm publish"