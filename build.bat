@echo off
setlocal
cd /d "%~dp0"

where npm >nul 2>nul || (
  echo Node.js/npm is required.
  exit /b 1
)
where cargo >nul 2>nul || (
  echo Rust/Cargo is required.
  exit /b 1
)

if not exist node_modules (
  call npm ci || exit /b 1
)

call npx tauri build --bundles msi
exit /b %ERRORLEVEL%
