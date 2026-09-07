# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Termlane is a lightweight desktop SSH terminal + SFTP + multi-server manager built on Tauri v2, Vue 3 and Rust.

Product direction: keep SSH / terminal / SFTP daily workflows fast, low-resource, direct and low-configuration. Do not turn the project into a feature-heavy terminal suite by default.

Current build metadata is `0.1.0`; the project is in 1.0 closure. Historical README/PLAN claims are not authoritative when they conflict with current code.

## Essential Commands

```bash
# Browser UI development (mock/fallback only)
npm run dev

# Full desktop app
npm run tauri:dev

# Frontend validation
npm test
npm run test:smoke
npx vue-tsc --noEmit
npm run build

# Rust validation
cd src-tauri
cargo check
cargo test

# Production bundle
npm run tauri:build
```

GitHub Actions runs the frontend and Rust quality gates on release. Do not mark a change validated when the latest relevant CI run is failing or still incomplete.

## Architecture

```text
Vue 3 WebView
  -> @tauri-apps/api ESM invoke/listen
  -> Tauri command boundary
  -> Rust ssh2 / portable-pty / keyring / filesystem
```

### Frontend

- Entry: `src/main.ts -> src/App.vue`.
- No Pinia/Vuex. Shared state is primarily in `src/composables/useAppState.js`.
- `src/utils/tauri.ts` is the single Tauri IPC/browser-mock boundary.
- `withGlobalTauri` is disabled. Do not reintroduce `window.__TAURI__`; use the bundled `@tauri-apps/api` ESM APIs.
- Browser mode is a UI/mock environment. It does not prove SSH, PTY, SFTP, Keyring or packaging correctness.
- TypeScript migration is gradual: utility modules are mostly `.ts`, many Vue components still use plain `<script setup>`.

### Rust backend

- `src-tauri/src/lib.rs`: Tauri builder, plugins and command registration.
- `commands.rs`: IPC validation and delegation.
- `ssh.rs`: SSH connection, host-key verification and PTY shell.
- `sftp.rs`: remote file operations.
- `local_pty.rs`: local shell via `portable-pty`.
- `config.rs`: non-secret connection config, OS Keyring credentials, recording metadata.
- `updater.rs`: release version check only; it is not a real installer.

## Credential Boundary

Desktop secrets have one authoritative storage path: the Rust OS Keyring backend.

Rules:

- Never persist `password` or `passphrase` in `connections.json`, Tab snapshots, localStorage, logs or ordinary frontend state snapshots.
- `src/utils/credentials.ts` removes transient secrets before ordinary persistence and resolves credentials at connection time.
- Historical `xterminal-pwd_<id>` plaintext localStorage values are migration input only; migrate them to the secure backend and delete the plaintext key.
- Historical `xterminal-*` localStorage keys and the `xterminal-pro` config/Keyring namespaces are migration input only after the Termlane rename; new data must use the Termlane namespace.
- Duplicating a connection must not duplicate its credentials.
- Browser mode uses `secure-store-browser.ts` AES-GCM fallback only because no OS Keyring exists there. Its sessionStorage-held key is XSS-extractable and must not be described as desktop-equivalent security.

## SSH Host-Key Trust Model

Current code uses OpenSSH `~/.ssh/known_hosts` semantics:

- port 22 -> normal `host` entry;
- non-default port -> `[host]:port` and `check_port`;
- known + matching -> connect;
- unknown -> return structured algorithm + SHA256 fingerprint, show it to the user, and require explicit “trust and connect” before writing known_hosts;
- mismatch -> hard reject; do not provide a silent overwrite path.

Do not weaken this flow to restore automatic TOFU.

## ProxyJump Boundary

1.0 currently does **not** support ProxyJump.

The historical `ssh_connect_jump` implementation only tested a jump-host `direct-tcpip` channel and then connected directly from the client to the target. That implementation and its UI entry are being removed because it was not a real jump connection.

Do not restore a `jump` command or UI until the target SSH session actually runs over the intermediate transport. Do not add a second SSH stack casually just to recover a checkbox feature.

## Window State / Updater

- Window position/size persistence uses official `tauri-plugin-window-state`; do not recreate the deleted custom WindowState IPC/JSON path.
- Updater currently performs version discovery only.
- Do not add simulated download/install progress.
- A real updater belongs to the release pipeline and must use Tauri's signed updater artifacts, public key and endpoint metadata.

## Dependency / Reuse Policy

Prefer mature official/ecosystem implementation for protocol, security and platform infrastructure, but lightweight is a higher-order constraint than “replace every local helper”.

Current decisions:

- Keep the small fixed-height `VirtualList` until requirements actually need a larger virtualization library.
- Keep the small split-resize composable until nested panes/accessibility/constraint complexity justifies a library.
- Existing xterm `fit/search/web-links` addons cover current proven needs; do not add WebGL/Unicode/clipboard addons without an actual requirement or measured problem.
- Do not introduce VueUse/Pinia merely for stylistic consistency.
- Remove unused dependencies when the lockfile can be updated and CI proves the change.

## Testing

- Vitest is the frontend unit-test framework.
- `npm run test:smoke` is a Vitest suite and must remain executable.
- Security boundaries added or changed should have focused tests.
- Browser mocks must use the same command names and argument shapes as native IPC where practical.
- Component existence or mock success does not count as native capability verification.

## Current Known Release Work

Tracked in:

- #1 foundation consolidation
- #2 UI/product interaction closure
- #3 1.0 release baseline

Important remaining release work includes real desktop flow verification, signed updater pipeline, dependency-security review, package/install verification, performance measurement and documentation consistency.

## Coding Conventions

- Conventional Commits.
- Chinese is preferred for durable project documentation and user-facing product text.
- Keep frontend IPC through `src/utils/tauri.ts` unless a direct official API is intentionally required.
- Do not introduce patch-style parallel implementations: when replacing infrastructure, remove the obsolete path.
- Do not claim a feature complete because a command/component/PLAN checkbox exists; inspect the full user-to-native path.
