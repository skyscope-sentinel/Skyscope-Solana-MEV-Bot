# Repository Structure Analysis – Skyscope-Solana-MEV-Bot

This document gives a bird’s-eye view of how the project **should** be organised after the recent GUI & security enhancements.  
Use it as a checklist to verify what is already in the repo and what still needs attention.

---

## 1. Root-level Layout

| Path | Purpose |
|------|---------|
| `Cargo.toml` | Rust workspace manifest (backend + trading engine) |
| `build-macos.sh` | One-click universal binary & DMG builder for macOS |
| `tauri.conf.json` | Tauri desktop-app configuration |
| `README.md` | Project overview, install & usage |
| `.github/workflows/` | CI for lint, tests, release & notarisation |
| `.gitignore` | Ignore targets, node_modules, build artefacts |
| `LICENSE` | OSS license (recommended MIT or Apache-2.0) |

> 🔍 **Quick check:** All of the above should exist at repo root. Remove duplicates (e.g. multiple Cargo.toml) and make sure no large artefacts (build/ or target/) are committed.

---

## 2. Rust Backend (`src/`)

```
src/
├── main.rs            <-- CLI & app entry point
├── security.rs        <-- Argon2 PIN hashing, retry limiter
├── keystore.rs        <-- XChaCha20-Poly1305 encrypted wallet store
├── authentication.rs  <-- Session manager, timeout
├── trading.rs         <-- Strategy engine & DEX adapters
└── lib.rs             <-- (optional) shared helpers
```

* All modules should be declared in `lib.rs` or `main.rs`.
* Ensure real DEX adapters replace any mocks before production.
* Unit tests live beside code or under `tests/`.

---

## 3. Desktop Shell (`src-tauri/`)

```
src-tauri/
├── src/
│   └── main.rs        <-- Tauri command handlers & IPC bridge
├── icons/             <-- App icons for macOS & Windows
└── tauri.conf.json    <-- symlink / copy from root
```

* Confirm the `tauri` crate versions match those in `Cargo.toml`.
* If notarisation/codesigning keys are local, exclude from repo.

---

## 4. Front-end (`frontend/`)

```
frontend/
├── package.json
├── tsconfig.json
└── src/
    ├── App.tsx
    ├── components/
    │   ├── WalletImport.tsx
    │   └── ... other reusable UI pieces
    ├── assets/        <-- logo, dark-mode tokens
    └── styles/        <-- Tailwind / CSS-in-JS files
```

* `yarn install` / `npm ci` should succeed with no audit-critical issues.
* Remove `node_modules/` from version control.

---

## 5. Build & Distribution

1. **`build-macos.sh`**  
   - Installs Rust toolchain, Node, Tauri CLI  
   - Creates universal (`x86_64 + arm64`) `.app` bundle  
   - Codesigns if certificates are present  
   - Produces a DMG

2. **CI workflow** (`.github/workflows/release.yml`)  
   - Runs the same script on macOS runner  
   - Uploads signed artefacts as GitHub Release

---

## 6. Documentation & Testing

* `docs/` (optional) for extended guides, architecture diagrams.
* `tests/` or module-level `#[cfg(test)]` suites for:
  - PIN auth edge-cases
  - Keystore encryption/decryption
  - Strategy profitability simulation

---

## 7. Cleanup Checklist

- [ ] No compiled artefacts (`target/`, `dist/`, `frontend/build/`) in Git.
- [ ] Remove placeholder code / mock DEX adapters once real ones land.
- [ ] Delete any `.DS_Store`, `Thumbs.db`, `*.log`.
- [ ] Ensure **one** `Cargo.lock` at workspace root (if needed).
- [ ] Verify `LICENSE` year & holder.

---

## 8. Next Actions

1. Compare this structure with current repo state.  
2. Add missing files/directories.  
3. Remove anything not listed here, unless intentionally added.  
4. Push changes & rerun CI.  

When all checkmarks are green, the repository will be clean, reproducible, and ready for onboarding contributors or distributing binaries.
