# CLAUDE.md

This file provides source-repository guidance for agents working on XTerminal Pro. When documentation conflicts with current code or build configuration, current code/config wins and the stale documentation must be corrected.

## Project Overview

XTerminal Pro is a lightweight desktop SSH terminal + SFTP + multi-server manager built with Tauri v2, Vue 3, Vite and Rust. The current authoritative application version is `0.1.0` (`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`).

Product direction: keep startup fast, memory use low, interaction direct, configuration minimal, and avoid feature/framework bloat. Prefer mature official or well-maintained implementations for generic infrastructure; reserve custom code for product-specific behavior.

## Essential Commands

```bash
# Browser dev mode; uses the browser/mock path, not real SSH/SFTP
npm run dev

# Full desktop runtime
npm run tauri:dev

# Production build
npm run tauri:build

# Frontend tests
npm test
npm run test:smoke
npm run test:coverage

# Type check
npx vue-tsc --noEmit

# Rust checks/tests (run from src-tauri/)
cargo check
cargo test
```

- Node.js >= 18.
- Rust must satisfy the Tauri plugin requirements; `tauri-plugin-window-state` requires Rust >= 1.77.2.
- Linux build dependencies include WebKitGTK/GTK/OpenSSL development packages.
- Windows builds use the MSVC Rust toolchain and OpenSSL environment configured by the project build environment.

## Architecture

```text
Vue 3 WebView --Tauri IPC/events--> Rust backend --native--> SSH / PTY / filesystem / keyring
```

### Frontend

- Entry: `src/main.ts` -> `src/App.vue`.
- State: no Pinia/Vuex. Shared application state is currently centered in `src/composables/useAppState.js` plus local component refs.
- IPC abstraction: `src/utils/tauri.ts` wraps Tauri invoke/listen behavior and provides browser mock behavior. Browser mode is not evidence that native SSH/SFTP works.
- `App.vue` lazy-loads secondary feature views; the terminal shell is a primary synchronous view.
- Browser secure-storage fallback uses AES-GCM with its key material available to the browser session. Treat it as a development/browser fallback, not equivalent to the desktop OS keyring.

### Rust backend

| Module | Responsibility |
| --- | --- |
| `lib.rs` | Tauri builder, plugins and command registration |
| `commands.rs` | IPC validation/delegation |
| `ssh.rs` | SSH connections, authentication and interactive PTY |
| `sftp.rs` | Remote/local file operations |
| `config.rs` | Connection config, keyring credentials and recording metadata |
| `local_pty.rs` | Local shell via `portable-pty` |
| `updater.rs` | Gitea release version check only; no in-app install yet |
| `utils.rs` | Shared Rust helpers/macros |

Window geometry persistence is handled by the official `tauri-plugin-window-state`. Do not reintroduce a parallel `window_state.json` implementation.

## Important Engineering Risks

- `ssh2` is synchronous. Avoid hiding blocking SSH work inside async functions without an explicit blocking/threading strategy.
- PTY/session lifecycle must be reviewed for thread/session cleanup and idle resource use before 1.0.
- Do not call a feature complete because a Vue component exists. Verify UI entry -> state -> IPC -> Rust/native behavior -> error path -> tests/manual validation.
- Keep dependencies deliberate. Do not add VueUse, virtual-list frameworks, split-pane frameworks, or a UI framework solely to replace small stable local helpers; add them only when the local requirement has actually outgrown the simple implementation.

## Testing

- Vitest 4.x + jsdom.
- Tests live in `src/__tests__/`.
- `npm run test:smoke` runs `src/__tests__/smoke-vitest.test.js`.
- Rust unit tests run with `cargo test` from `src-tauri/`.
- Browser mocks are useful for frontend behavior but cannot validate SSH/SFTP/PTY native integration.
- For changes to native desktop flows, add Rust/integration coverage where practical and record a real Tauri runtime validation path.

## Coding Conventions

- Use Conventional Commits (`feat:`, `fix:`, `refactor:`, `chore:`, `docs:`, `test:`).
- Prefer focused changes over compatibility shims and duplicated implementations.
- Remove obsolete code after replacing an implementation; do not leave two competing paths active.
- Keep the current gradual TypeScript migration direction; do not perform broad mechanical rewrites unrelated to the task.
- Prefer Chinese comments/doc text where project-facing explanation is needed; identifiers and protocol/library names remain their canonical names.

## Security Architecture

- CSP is enabled in `src-tauri/tauri.conf.json` and restricts scripts/styles/images/connections/fonts. Keep it enabled and narrow permissions rather than disabling it to fix integration problems.
- Desktop credentials use the OS keyring through the Rust `keyring` crate. Do not persist passwords in connection JSON/localStorage.
- SSH host keys are currently checked against `~/.ssh/known_hosts` using TOFU behavior: known matching keys are accepted, unknown hosts are added on first use, and mismatches are rejected. Before 1.0, audit non-default-port handling and provide a deliberate first-use fingerprint/host-trust UX instead of treating the current implementation as final.
- Tauri capability `default` currently grants `core:default` to the `main` window. When adding plugins or frontend plugin commands, grant only the permissions actually required.
- `withGlobalTauri` is currently enabled. Do not expand globally exposed capabilities without a security reason.

## Update Strategy

The current updater path only checks the latest Gitea release. It must not simulate downloading/installing updates.

Tauri's official updater requires signed updater artifacts, a durable signing private key, embedded public key and a compatible update endpoint/static JSON. Implement the official updater only together with that release/signing pipeline; do not ship an unsigned or placeholder updater flow.

## Key Documentation

- `docs/ARCHITECTURE.md`
- `docs/API.md`
- `docs/CODING-STANDARDS.md`
- `docs/TESTING.md`
- `docs/PLAN.md` — historical plan; verify against current code before treating checkboxes as current status
- `docs/SPEC.md`
