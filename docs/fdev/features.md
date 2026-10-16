# Feature-source audit

The active application is Codex Rust. These are scoped implementation reviews,
not claims that every imported file survived the merge or that every feature
has been integrated. The other source trees remain in place and outside the
`codex-rs` workspace. No second agent runtime is started by FDev.

| Source | Implementation inspected in this import | Codex overlap | Decision / status |
| --- | --- | --- | --- |
| OpenCode | `packages/core/src/provider.ts`, `packages/ai/src/provider.ts` | `model-provider-info`, `models-manager`, TOML profiles and TUI model selection | Keep provider identity separate from endpoint/model selection. FDev's new `provider config` helper uses Codex's existing format; it does not import the TypeScript runtime. |
| Gemini CLI | `packages/core/src/scheduler/confirmation.ts`, scheduler policy and cancellation paths | `core` tool orchestration, `execpolicy`, approval events | Retain Codex approvals and interruption handling. No second scheduler. Native Gemini inference still needs an adapter. |
| Aider | `aider/repomap.py` (token budget, tag cache, repository map) | file search, Git tools, context compaction | Add a bounded Git-tracked filename map. Implemented independently in Rust; no semantic ranking, tag extraction, Python dependency, or claim of Aider parity. |
| Cline | `sdk/packages/core/src/runtime/tools/tool-approval.ts`, task store locations | runtime approval requests, plan/goal extensions, rollout recovery | Keep one approval owner in Codex. Desktop IPC and browser integration are not ported. |
| Goose | `crates/goose/src/agents/extension_manager/mod.rs`, `providers/custom_provider_config.rs` | `rmcp-client`, MCP extension, environment-backed provider credentials | Reuse existing MCP support. Do not add Goose's parallel extension lifecycle or credential store. |
| Continue | `core/context/providers/RepoMapContextProvider.ts` | contextual file input, repository navigation, model profiles | Explicit on-demand context command inspired by Continue and Aider. Symbol signatures and IDE context providers remain deferred. |
| OpenHands | `src/api/conversation-workspace.ts` | working directory, worktrees, local/remote executor boundary | Retain Codex workspace/executor ownership. Do not require the Canvas frontend or Python Agent Server. |

## Implemented feature port

`fdev context-map --path . --max-files 200` lists Git-tracked filenames from the
selected directory. Git's index determines membership; untracked files are
excluded. Names are escaped to prevent terminal-control injection. Output is
bounded to 1,000 files and approximately 64 KiB; truncation is explicit. Git is
reaped even when the reader stops early. No file contents or symbol database
are read. Use the output as explicit prompt context when useful.

Native Termux module tests passed: tracked files versus untracked files,
truncation/invalid limits, non-repository failure, and control-character names.
The full executable command still needs CI validation.

## License boundary

Original license texts and Codex's NOTICE remain under `licenses/` and are
packaged with artifacts. The new context helper is an original Rust
implementation of a general file-inventory idea; no Aider or Continue code was
copied. The provider helper similarly adapts the existing Codex configuration,
without copying another provider implementation. This is not a completed
transitive dependency license audit. Do not remove reference sources until
provenance, overwritten files, and their licenses have been reviewed.
