# FDev recovery specifications

- FDEV-001: The production entry point is `codex-rs/cli`, built as `fdev`,
  with the existing Codex runtime and TUI. Help and version identify the fork.
- FDEV-002: FDEV_HOME selects the configuration root; CODEX_HOME is a
  compatibility override. New installations use ~/.fdev. Invalid explicit
  paths must fail rather than silently use another directory.
- FDEV-003: Provider setup uses the existing model-provider registry, validates
  endpoint and credential references, and does not expose secrets.
- FDEV-004: Unsupported Android sandbox facilities must never silently grant
  unrestricted execution.

Implementation and verification status lives in docs/fdev/recovery.md.
