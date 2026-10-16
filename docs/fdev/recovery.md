# FDev reconstruction: evidence and milestones

FDev is one Codex-based coding agent. The active Rust workspace is `codex-rs`;
its `cli`, `core`, and `tui` own the executable, runtime, and terminal UI.
Other merged source trees are reference material, outside this workspace.
Do not build the root Cargo/npm/Bazel workspaces to build FDev.

## Baseline audit

- Branch: `fdev/codex-recovery`, starting at `3de37e0b`.
- PR #2 was closed unmerged. Its Codex-first architecture remains the intended
  direction, not a completed implementation.
- PR #3 merged as `75ee4d4d`, removing 222 inherited workflows. Preserve that
  cleanup; do not restore the upstream workflow collection.
- PR #1 later introduced a separate Python chat client and packaging workflow.
  Its sources remain for reference; its workflow is now `.yml.disabled` so
  it cannot ship a different application under the same executable name.
- Compared the imported Codex Cargo manifest, CLI main, core manifest, and
  TUI manifest with upstream `2bf2bdf82c33f4c66ba3f0767b8e4b6a933aac40`:
  these files were byte-identical before FDev edits. Cargo.lock differed;
  full provenance and dependency-license audit remain pending.
- `cargo metadata --no-deps --format-version 1` passed on native
  `aarch64-linux-android`, rustc 1.99.0 (Termux).
- Initial offline check failed because patched crossterm was not cached.
  Online dependency fetching works. This is not a source compilation failure.
- Prior main CI run 38935384245 passed Linux CLI/TUI `cargo check`.
  A check is not evidence of a linked executable or successful interaction.

## Milestone 1: establish the executable and build loop

The Codex CLI Cargo target is now `fdev`, with FDev help/version identity.
Internal crate names and helper protocol names remain unchanged. Upstream
Cargo integration-test callers resolve `codex` to `fdev`; Bazel reference
names are left intact. Default workspace builds select the CLI.

CI builds Windows x86_64, Linux x86_64, and Linux ARM64 on native runners.
It runs clippy, focused unit tests, release builds, a CLI branding integration
test, and help/version smoke tests. Artifacts include upstream license texts.
No macOS builds or release publishing are enabled. Linux ARM64 is explicitly
not a Termux artifact. These jobs are configured, not yet verified green.

## Remaining milestones

1. Fix actual CI/native compiler and linker failures; verify CLI and TUI startup.
2. Add isolated FDev configuration, provider validation/diagnostics, safe update
   behavior, and terminal branding while preserving Codex configuration formats.
3. Audit seven feature-source implementations and licensing, selecting only
   capabilities absent from Codex. Port features with behavioral tests.
4. Implement missing provider protocols behind the existing Responses-centric
   runtime, including streaming/tool-call tests and real provider validation.
5. Resolve native Android dependencies, PTY/process behavior, TLS, and sandbox
   policy. Verify on this real Termux device before advertising support.
6. Validate filesystem edits and approved commands end-to-end; document actual
   installation paths and package verified Windows/Linux artifacts.

No successful real model interaction or native FDev executable has been
verified yet. Do not treat this document or a CI definition as proof of support.
