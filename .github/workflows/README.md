# FDev build workflow

`fdev-codex-ci.yml` is the active product workflow. It uses the pinned Codex
Rust toolchain and builds the `fdev` executable from `codex-rs/cli` on Windows
x86_64, Linux x86_64, and native Linux ARM64 runners. It performs formatting,
clippy, focused unit/integration tests, release linking, CLI smoke checks, and
artifact uploads including license texts. It never publishes a release.

`fdev-core-build.yml.disabled` preserves the old Python bootstrap workflow as
reference; GitHub does not execute it. PR #3 removed the inherited upstream
workflow collection. Do not restore it or add macOS builds. Linux ARM64
artifacts must never be labeled Termux/Android artifacts.

Runtime and build status are tracked in `docs/fdev/recovery.md`. A workflow
configuration alone is not proof that a platform is supported.
