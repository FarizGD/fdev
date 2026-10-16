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
- Exact imported Codex baseline: upstream commit
  `4eb4a65407d98cebc4c83bf13be440b9fbc78a7c`. The complete imported
  `codex-rs` Git tree matches upstream tree
  `4f2abde84b6caf920c44bb9e5e897cc15015cc76` (8,737 files).
  No restoration is needed inside that tree. Root-level collisions remain
  outside the active workspace; this comparison does not audit other imports.
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

## Milestone 2: configuration and bounded repository context

Work continues on `main` at the user's request; the development branch was
fast-forwarded into main and pushed. No history was rewritten.

- Added FDEV_HOME / ~/.fdev with CODEX_HOME compatibility and strict invalid-path
  handling. Native Termux TDD: the added tests failed before implementation;
  all 7 home-directory tests passed afterward.
- Added `provider list` and `provider config`, generating Responses-compatible
  configuration with URL/key-reference/context-limit validation. Runtime TOML
  deserialization and CLI tests are included; CI results are pending.
- Added `context-map`, a bounded, escaped Git file inventory inspired by Aider
  and Continue. Four real-Git tests passed natively after failing before the
  implementation. The same source module was compiled with rustc --test and
  the Cargo-built tempfile dependency; full CLI integration is still pending.
- Prevented the CLI/TUI update path from installing upstream Codex packages.
  Automatic updates remain unavailable until FDev distribution is verified.
- Added a local Responses SSE integration test driving the actual CLI,
  patch tool, and preapproved read/echo shell command in a disposable workspace.
  Only the model service is mocked. This is not a sandbox-isolation test.

See [feature comparison](features.md) and [provider setup](providers.md).

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
