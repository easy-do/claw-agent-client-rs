#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
COMPOSE_DIR="$PROJECT_DIR/compose"

echo "========================================"
echo "  Claw Agent Client Rs Build Script"
echo "========================================"
echo ""

# 检查 Rust 是否已安装
if ! command -v rustc &> /dev/null; then
    echo "[1/4] Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source "$HOME/.cargo/env"
else
    echo "[1/4] Rust already installed"
fi

echo "[2/4] Checking dependencies..."

echo "[3/4] Building project..."
cd "$PROJECT_DIR"
cargo build --release

echo "[4/4] Copying files to compose..."

# 创建 compose 目录
mkdir -p "$COMPOSE_DIR/config"
mkdir -p "$COMPOSE_DIR/scripts"

# 复制编译产物
if [ -f "target/release/claw-agent-client-rs" ]; then
    cp target/release/claw-agent-client-rs "$COMPOSE_DIR/"
    echo "Binary copied to: $COMPOSE_DIR/claw-agent-client-rs"
fi

# 复制配置文件
if [ -f "$PROJECT_DIR/config/agent.yml" ]; then
    cp "$PROJECT_DIR/config/agent.yml" "$COMPOSE_DIR/config/"
    echo "Config copied to: $COMPOSE_DIR/config/agent.yml"
fi

# 复制元数据文件
if [ -f "$PROJECT_DIR/config/metadata.json" ]; then
    cp "$PROJECT_DIR/config/metadata.json" "$COMPOSE_DIR/config/"
    echo "Metadata copied to: $COMPOSE_DIR/config/metadata.json"
fi

# 复制安装脚本
if [ -f "$SCRIPT_DIR/../installs/macos_install.sh" ]; then
    cp "$SCRIPT_DIR/../installs/macos_install.sh" "$COMPOSE_DIR/scripts/"
    echo "Install script copied to: $COMPOSE_DIR/scripts/macos_install.sh"
fi

echo ""
echo "========================================"
echo "Build completed!"
echo "========================================"
echo ""
echo "Output location: $COMPOSE_DIR"
echo ""
echo "Next steps:"
echo "  1. Configure your agent.yml in config folder"
echo "  2. Run: cd $COMPOSE_DIR && sudo ./scripts/macos_install.sh"
echo ""
