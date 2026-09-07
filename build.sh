#!/bin/bash
# Termlane 构建脚本
# 用法: ./build.sh [dev|build]

set -e

source "$HOME/.cargo/env"

MODE="${1:-build}"

echo "=== Termlane 构建 ==="
echo "模式: $MODE"
echo ""

# 检查系统依赖
check_deps() {
    echo "检查系统依赖..."
    MISSING=""
    for pkg in pkg-config libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev; do
        if ! dpkg -l | grep -q "$pkg"; then
            MISSING="$MISSING $pkg"
        fi
    done
    if [ -n "$MISSING" ]; then
        echo "⚠️  缺少依赖:$MISSING"
        echo ""
        echo "请运行以下命令安装:"
        echo "  sudo apt install -y$MISSING"
        echo ""
        echo "安装完成后重新运行此脚本。"
        exit 1
    fi
    echo "✅ 系统依赖齐全"
}

# 前端构建
build_frontend() {
    echo "构建前端..."
    npm run build
    echo "✅ 前端构建完成"
}

# Tauri 构建
build_tauri() {
    echo "构建 Tauri 应用..."
    if [ "$MODE" = "dev" ]; then
        cargo-tauri dev
    else
        cargo-tauri build
        echo ""
        echo "✅ 构建完成！"
        echo "输出目录: src-tauri/target/release/"
        echo ""
        ls -la src-tauri/target/release/bundle/ 2>/dev/null || echo "bundle 目录将在完整构建后生成"
    fi
}

check_deps
build_frontend
build_tauri
