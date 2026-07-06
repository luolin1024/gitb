#!/usr/bin/env bash
# gitb one-line installer for macOS / Linux
# Usage:  curl -fsSL https://github.com/luolin1024/git-batch/raw/main/install.sh | bash
set -euo pipefail

VERSION="v0.3.0"
REPO="luolin1024/git-batch"
BINARY_NAME="gitb"

# Detect platform and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin) PLATFORM="macos" ;;
  Linux)  PLATFORM="linux" ;;
  *) echo "❌ Unsupported OS: $OS (use PowerShell script for Windows)"; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH_NAME="x86_64" ;;
  arm64|aarch64) ARCH_NAME="aarch64" ;;
  *) echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
esac

FILE="gitb-${ARCH_NAME}-${PLATFORM}"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${FILE}"

# Choose install directory
if [ -w "/usr/local/bin" ]; then
  DEST="/usr/local/bin/${BINARY_NAME}"
elif [ -d "$HOME/.local/bin" ] || mkdir -p "$HOME/.local/bin" 2>/dev/null; then
  DEST="$HOME/.local/bin/${BINARY_NAME}"
  # Ensure ~/.local/bin is in PATH
  case ":$PATH:" in
    *":$HOME/.local/bin:"*) ;;
    *) echo "⚠️  $HOME/.local/bin is not in your PATH." ;;
  esac
else
  DEST="$PWD/${BINARY_NAME}"
  echo "⚠️  Cannot write to /usr/local/bin or ~/.local/bin, installing to current directory."
fi

echo "⬇️  Downloading gitb ${VERSION} (${ARCH_NAME}-${PLATFORM})..."
curl -fsSL -o "$DEST" "$URL"
chmod +x "$DEST"

echo "✅ Installed to: $DEST"
echo "🚀 Run 'gitb --version' to verify."
