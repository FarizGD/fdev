# FDev: Codex-first architecture

FDev uses the Apache-2.0 licensed Codex Rust agent as its primary engine, CLI and TUI.

## Authoritative source
- Engine: `codex-rs/core`
- CLI: `codex-rs/cli`
- TUI: `codex-rs/tui`
- Rust workspace: `codex-rs/Cargo.toml`

The repository root currently has conflicting manifests from other upstreams. Do not build from the root or assume the eight source trees are one integrated application.

## Roadmap
1. Validate baseline Codex CLI/TUI compilation and fix failures.
2. Rebrand command, executable, package and TUI as FDev.
3. Extend provider configuration for Ollama and OpenAI-compatible services.
4. Port selected features from other upstreams as reviewed, tested changes.
5. Test Windows, Linux, and Android/Termux, including native Bionic compatibility.
6. Preserve license texts, notices and upstream author attribution.

No macOS builds requested. Windows/Linux CI alone is not evidence that native Termux works.

## Local baseline
```sh
cd codex-rs
cargo metadata --no-deps --format-version 1
cargo check -p codex-cli --locked
cargo check -p codex-tui --locked
```
