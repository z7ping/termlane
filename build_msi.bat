@echo off
set HTTP_PROXY=http://192.168.31.87:7897
set HTTPS_PROXY=http://192.168.31.87:7897
set OPENSSL_DIR=D:\vcpkg\installed\x64-windows
set OPENSSL_STATIC=1
set PATH=D:\node-v22.16.0-win-x64;D:\rustup\bin;D:\cargo\bin;D:\vcpkg;%PATH%
cd /d F:\01-ai-gen-workspaces\termlane
echo Building MSI... %TIME%
npx tauri build
echo Done %ERRORLEVEL% %TIME%
