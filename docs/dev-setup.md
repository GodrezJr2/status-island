# Dev setup

## Home PC (the build machine; admin available)

1. Install **Visual Studio Build Tools 2022** with the workload "Desktop development with C++".
2. Install Rust from https://rustup.rs (default MSVC host), then run `rustup target add aarch64-pc-windows-msvc`.
3. Install Node 22+ and Git, plus the GitHub CLI (`gh auth login`).
4. WebView2 ships with Windows 11. On Windows 10, install the Evergreen runtime.
5. For real-run checks, have Claude Code, Codex and OpenCode installed and signed in, and the ntfy app on your phone.
6. Check it: `cargo test -p si-core`. Expected: 4 passed.

---

## Work laptop (no admin)

Verified on the owner's company PC on 2026-10-18.

## Rust

The MSVC C++ build tools are not installed, and installing them needs admin. Use the **GNU toolchain** instead. It ships its own linker and installs entirely per user.

1. Download the installer: `https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-gnu/rustup-init.exe`
2. Verify it. This build is not Authenticode-signed, so compare its SHA-256 with `…/rustup-init.exe.sha256` from the same folder. In PowerShell, read the `.sha256` response as text, not bytes.
3. Install:
   ```powershell
   rustup-init.exe -y --profile minimal --default-host x86_64-pc-windows-gnu --default-toolchain stable
   ```
4. Open a new shell, or run `$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"`, then check `rustc --version`. Result on 2026-10-18: `rustc 1.99.1`.

crates.io works through the company network with no extra settings (tested with `serde_json`).

### Blocker for Task 3 onward (found 2026-10-18)

The GNU toolchain lacks two tools:
- **`dlltool.exe`**, needed by crates that link with `raw-dylib`: `windows-sys` 0.61, `getrandom` 0.4. Tauri depends on these.
- **A C compiler**, needed by `rusqlite`'s `bundled` feature.

So `si-core` stays pure Rust (Tasks 1, 2, 4, 5 and 6 build fine). Tasks 3 (OpenCode via SQLite) and 7+ (Tauri) need one of these, all without admin:

1. **Recommended:** ask IT to install **Visual Studio Build Tools 2022 → "Desktop development with C++"** (free, official). Then run `rustup default stable-x86_64-pc-windows-msvc`. This matches the CI and release targets exactly.
2. Portable **llvm-mingw** (zip from github.com/mstorsjo/llvm-mingw, unpack per user, add `bin` to PATH). It provides `dlltool` and `clang`. Antivirus may block the zip, as it did Electron's.
3. Build only on GitHub Actions (`windows-latest` has MSVC). This always works, but each change takes a few minutes to test.

To keep Task 3 moving without any of these, OpenCode could be read through Node's built-in `node:sqlite` from a tiny helper. That trades one dependency for another, so decide it when Task 3 starts.

**Release builds** for x64 and ARM64 use the MSVC targets on GitHub Actions (`windows-latest`), as described in plan Task 13. Local GNU builds are for development only.

## Node / npm

The company network inspects TLS. Use the Windows certificate store; never turn off certificate checks:

```powershell
$env:NODE_OPTIONS = '--use-system-ca'
```

The Electron binary download is blocked locally (EPERM, probably antivirus). This project doesn't use Electron.

## WebView2

The runtime is already installed (`C:\Program Files (x86)\Microsoft\EdgeWebView\Application`).
