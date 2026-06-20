@echo off
set HTTP_PROXY=http://192.168.31.87:7897
set HTTPS_PROXY=http://192.168.31.87:7897
set OPENSSL_DIR=F:\gitea-cache\vcpkg\installed\x64-windows
set OPENSSL_STATIC=1
set RUSTUP_HOME=F:\gitea-cache\rustup
set CARGO_HOME=F:\gitea-cache\cargo
set CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
set PATH=F:\gitea-cache\node\node-v22.23.0-win-x64;F:\gitea-cache\cargo\bin;F:\gitea-cache\rustup\bin;F:\gitea-cache\vcpkg;%PATH%
cd /d F:\01-ai-gen-workspaces\xterminal-pro
npx tauri build
