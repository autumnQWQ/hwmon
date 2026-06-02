#!/usr/bin/env bash
# hwmon installer — macOS / Linux
set -e

BINARY="hwmon"
INSTALL_DIR="${HW_INSTALL_DIR:-/usr/local/bin}"
CONFIG_DIR="${HOME}/.config/hwmon"

RED='\033[31m'; GREEN='\033[32m'; YELLOW='\033[33m'; NC='\033[0m'

echo "╔══════════════════════════════════════╗"
echo "║   hwmon — 极简硬件监控 安装程序     ║"
echo "╚══════════════════════════════════════╝"
echo ""

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"
case "$OS" in
    Darwin)  PLATFORM="macOS" ;;
    Linux)   PLATFORM="Linux" ;;
    *)       echo -e "${RED}Unsupported OS: $OS${NC}"; exit 1 ;;
esac
echo "📋 Platform: $PLATFORM ($ARCH)"

# Find binary in distribution
SCRIPT_DIR="$(cd "$(dirname "$0")" 2>/dev/null && pwd)"
if [ -f "$SCRIPT_DIR/$BINARY" ]; then
    SRC="$SCRIPT_DIR/$BINARY"
elif [ -f "./$BINARY" ]; then
    SRC="$(pwd)/$BINARY"
else
    echo -e "${RED}Error: $BINARY binary not found in package${NC}"
    echo ""
    echo "  Looked in:"
    echo "    $SCRIPT_DIR/"
    echo "    $(pwd)/"
    echo ""
    echo "  This script must be in the same folder as the 'hwmon' binary."
    echo "  If you downloaded a .tar.gz, extract the WHOLE archive, not just this script:"
    echo "    tar xzf hwmon-*.tar.gz"
    echo "    cd into the extracted folder"
    echo "    bash install.sh"
    echo ""
    echo "  If you have the Windows source .zip — that's source code only."
    echo "  You need the darwin-arm64.tar.gz for macOS."
    exit 1
fi

# Check for existing install
if [ -f "$INSTALL_DIR/$BINARY" ]; then
    EXISTING_VER="$($INSTALL_DIR/$BINARY --version 2>/dev/null || echo "unknown")"
    echo "⚠️  Existing installation found: $EXISTING_VER"
    read -p "Overwrite? [Y/n] " -n 1 -r; echo
    if [[ ! $REPLY =~ ^[Yy]?$ ]]; then
        echo "Aborted."
        exit 0
    fi
fi

# Install — try without sudo first, fall back if needed
echo "📦 Installing to $INSTALL_DIR/$BINARY..."
mkdir -p "$INSTALL_DIR" 2>/dev/null || true
if cp "$SRC" "$INSTALL_DIR/$BINARY" 2>/dev/null && chmod +x "$INSTALL_DIR/$BINARY" 2>/dev/null; then
    : # installed without sudo
else
    echo "  (sudo required for $INSTALL_DIR)"
    sudo mkdir -p "$INSTALL_DIR" 2>/dev/null || true
    sudo cp "$SRC" "$INSTALL_DIR/$BINARY"
    sudo chmod +x "$INSTALL_DIR/$BINARY"
fi

# Verify
if command -v "$BINARY" >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Installation successful!${NC}"
    echo ""
    echo "  Usage:"
    echo "    hwmon              # Single snapshot"
    echo "    hwmon --watch       # Real-time monitoring"
    echo "    hwmon --json        # JSON output"
    echo "    hwmon --help        # All options"
    echo ""
    $INSTALL_DIR/$BINARY --version 2>/dev/null || true
else
    echo -e "${YELLOW}⚠️  Installed but not in PATH. Add to your shell:${NC}"
    echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
fi
