# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

XTerminal Pro — lightweight SSH terminal + SFTP + multi-server manager built on Tauri v2. Desktop app (~30MB) using Vue 3 + Vite frontend with a Rust backend (ssh2-rs + tokio). Full-featured: PTY shell, file management, connection monitoring, session recording, port forwarding, batch commands, cron scheduler.

## Essential Commands

```bash
# Browser dev mode (no Tauri, uses mock backend at localhost:1420)
npm run dev

# Full Tauri desktop app
npm run tauri:dev

# Production build
npm run tauri:build          # Linux (.deb) + Windows (.msi) + macOS (.dmg/.app)

# Testing
npm test                     # vitest run (all tests)
npm run test:watch           # vitest watch mode
npm run test:coverage        # with coverage report
cargo test                   # Rust tests (from src-tauri/)

# Type checking
npx vue-tsc --noEmit        # type-check all TypeScript + Vue files

# Format / lint
npx prettier --check src/    # (not configured; add if needed)
```

- Node.js >=18, Rust >=1.70, system deps: `libwebkit2gtk-4.1-dev libgtk-3-dev libssl-dev` (Linux)
- Windows: MSVC Build Tools + Rust; the vendored openssl build needs `perl` on PATH
- **Note:** `typescript`, `vue-tsc`, and `@vue/tsconfig` are in `devDependencies` but may not be installed -- run `npm install` first

## Architecture

```
Frontend (Vue 3 WebView) --Tauri IPC--> Rust Backend --Native--> SSH/SFTP/Keyring
```

### Frontend (Vue 3 + Vite + TailwindCSS)
- **Entry:** `src/main.ts` -> `src/App.vue`
- **State management:** NO Pinia/Vuex. Custom composable `src/composables/useAppState.js` + reactive refs in App.vue. Global state lives on `App.vue` (connections, tabs, viewMode, activeConnectionId, activeTabId) and flows down as props/emits.
- **IPC:** `src/utils/tauri.ts` wraps Tauri `invoke()` and `listen()`. Auto-detects `!isTauri()` and switches to browser mock mode (mock responses for all commands, no real SSH). All other code imports `invoke`/`listen`/`isTauri` from this one file.
- **18 async-loaded views** via `defineAsyncComponent()` in App.vue (lazy loading), switched by `viewMode`.
- **Secure storage fallback:** `src/utils/secure-store-browser.ts` provides AES-256-GCM encryption for browser mode (key in sessionStorage, ciphertext in localStorage). In Tauri desktop, the Rust backend uses OS keyring instead.

### Rust Backend (src-tauri/src/)

| Module | File | Purpose |
|--------|------|---------|
| `lib.rs` | Command registration hub | All `#[tauri::command]` handlers registered via `generate_handler![]` |
| `commands.rs` | Input validation + delegation | Thin layer: validates args, then calls into `ssh`/`sftp`/`config` modules |
| `ssh.rs` | SSH connections + PTY | `ssh2`-based: `connect()`, `connect_with_key()`, `connect_jump()`, PTY shell with reader thread |
| `sftp.rs` | File operations | Shell-based remote ops (ls/cat/mkdir/rm/chmod) via `ssh::execute()` + base64-encoded file transfer |
| `config.rs` | Persistence | JSON file I/O for connections, settings, window state, recordings. Credentials go to OS keyring via `keyring` crate. |
| `local_pty.rs` | Local terminal | `portable-pty`-based local shell (no SSH needed) |
| `updater.rs` | App updates | Checks Gitea releases API for new versions |
| `utils.rs` | Macro helpers | `lock!()` macro for `Mutex` access with poison recovery |

**Threading anti-patterns to be aware of:**
- `ssh.rs` uses synchronous `TcpStream` + `ssh2::Session` inside `async` Tauri commands -- this blocks Tokio threads
- `sftp.rs` creates a second `tokio::runtime::Runtime` (`static RUNTIME`) and calls `block_on()` inside the existing Tauri runtime -- runtime-in-runtime, can exhaust thread pool
- PTY reader thread uses busy-polling with `thread::sleep(5ms)` -- 200 wakes/sec even idle

**Data flow pattern (PTY shell):**
1. Frontend calls `invoke('ssh_start_shell', {host, port, username, password, cols, rows})`
2. Rust creates SSH session, allocates PTY, spawns reader thread
3. Reader thread streams output via `app.emit("shell_output", {session_id, data})`
4. Frontend listens: `listen('shell_output', (e) => terminal.write(e.payload.data))`
5. User input: `invoke('ssh_shell_input', {session_id, data})` -> Rust writes to PTY channel

### Key globals (Rust)
- `SESSIONS: LazyLock<Mutex<HashMap<String, ssh2::Session>>>` -- active SSH connections
- `PTY_SHELLS: LazyLock<Mutex<HashMap<String, ShellSession>>>` -- active PTY shell channels
- `MONITOR_SESSIONS`, `SFTP_SESSIONS` -- similar pattern for monitors and SFTP handles
- All use the `lock!()` macro from `utils.rs`

## Testing

- **Framework:** Vitest 4.x with `jsdom` environment, `globals: true`
- **Setup:** `src/__tests__/setup.js` mocks Tauri APIs (`@tauri-apps/api`, `@tauri-apps/plugin-secure-storage`)
- **Pattern:** Test files are `.js` (not `.ts`), live in `src/__tests__/`, import from `../utils/<module>.ts`
- **Coverage gap:** 10 out of 23 utils have tests; zero Vue component tests exist. Target what you change.
- **Smoke test script** `test:smoke` references `src/__tests__/smoke.test.js` which **does not exist** -- the `smoke-vitest.test.js` file uses vitest, not plain node

## Coding Conventions

- **Git:** Conventional Commits (`feat:`, `fix:`, `refactor:`, `chore:`, `docs:`, `test:`)
- **Rust:** Document structs/commands with `///` doc comments (Chinese preferred per CODING-STANDARDS.md)
- **Frontend imports:** Use `@/` alias -> `./src/`. Despite the TS migration, most `.vue` files still use `<script setup>` (no `lang="ts"`) and import with `.js` extensions (e.g. `import { invoke } from './utils/tauri.js'`). This works at build time via Vite resolution but type-checking will miss these files.
- **i18n:** `src/utils/i18n.ts` -- Chinese/English, accessed via `t('key')` pattern
- **Component size:** Several large components: Onboarding.vue (628 lines), TerminalPanel.vue (523), SftpPanel.vue (516), ConnectionDialog.vue (~400). Consider splitting when adding features.

## Security Architecture

- **CSP:** `tauri.conf.json` has `"csp": null` (disabled). The `index.html` has no CSP meta tag. The CHANGELOG and README claim CSP was added, but it's currently null. Any XSS has unrestricted access.
- **SSH host keys:** NOT verified -- `ssh.rs` never calls `set_known_hosts_check()`. All connections trust any host (MITM-vulnerable).
- **Credentials:** OS keyring via `keyring` crate (Rust) or AES-256-GCM in sessionStorage (browser fallback). AES key in sessionStorage is documented as XSS-extractable.
- **Capabilities:** `capabilities/default.json` grants only `core:default`. No filesystem/shell/dialog permissions declared. `withGlobalTauri: true` exposes `window.__TAURI__` globally.

## Key Documentation

- `docs/ARCHITECTURE.md` -- detailed system design with data flow diagrams
- `docs/API.md` -- full Rust command signatures with frontend call examples
- `docs/CODING-STANDARDS.md` -- comment conventions (JSDoc + Rust doc)
- `docs/TESTING.md` -- TDD workflow and test templates
- `docs/PLAN.md` -- development plan with phase tracking
- `docs/SPEC.md` -- product specification
