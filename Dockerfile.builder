# XTerminal Pro 构建镜像
# 基于 debian:bookworm，预装 Node.js 22 + Rust + Tauri 全部系统依赖
# 构建一次，CI 直接用，不用每次装环境
#
# 构建命令（在 Runner 宿主机上）：
#   podman build -f Dockerfile.builder -t xterminal-builder .
#
# 注意：xterminal-lite 和 xterminal-pro 共用同一个 xterminal-builder 镜像
# 如果 xterminal-pro 需要额外依赖，在此文件中添加后重新构建

FROM debian:bookworm

ENV DEBIAN_FRONTEND=noninteractive

# 系统依赖 + Node.js 22 + 常用工具
RUN apt-get update && apt-get install -y --no-install-recommends \
    # Tauri 系统依赖
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    librsvg2-dev \
    libayatana-appindicator3-dev \
    libsoup-3.0-dev \
    libjavascriptcoregtk-4.1-dev \
    libssl-dev \
    # ssh2 vendored-openssl 需要
    perl \
    # keyring crate 需要
    libdbus-1-dev \
    # 构建工具
    build-essential \
    curl \
    wget \
    pkg-config \
    # base64 工具（SFTP 传输用）
    coreutils \
    # Windows 交叉编译工具链
    gcc-mingw-w64-x86-64 \
    # Node.js 22
    && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Rust + Tauri CLI + Windows 交叉编译 target
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && . $HOME/.cargo/env \
    && rustup target add x86_64-pc-windows-gnu \
    && cargo install tauri-cli

ENV PATH="/root/.cargo/bin:${PATH}"

# 验证安装
RUN node --version && npm --version && rustc --version && cargo --version && cargo tauri --version
