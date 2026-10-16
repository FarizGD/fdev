# FDev core v0.1.0-alpha

This is an **isolated bootstrap**, not a merge of the eight original agent runtimes.
The top-level Cargo, npm, pnpm, Bazel, and Python workspaces remain conflicting upstream remnants.

## Run
- Termux: `pkg install python && python fdev.py --model qwen2.5-coder:7b`
- Linux / Windows: `python fdev.py --model qwen2.5-coder:7b`
- One-shot: `python fdev.py --file src/main.py "Explain this code"`

Default endpoint: `http://127.0.0.1:11434/v1/chat/completions` (Ollama).
Configure `FDEV_BASE_URL`, `FDEV_API_KEY`, and `FDEV_MODEL` or CLI flags.
For remote Ollama use your own trusted URL. Do not publish API keys.

## Builds
GitHub Actions produces a Windows executable and a Linux executable using PyInstaller.
Termux artifact is a **Python source distribution**, not a native compiled Android executable.
CI only tests this core, not the imported upstream workspaces.

## Current capabilities
One-shot prompts, interactive chat, OpenAI-compatible chat completion endpoints,
optional project file attachment (read only, 100 KB/file), and basic tests.

## Not implemented
True multi-agent orchestration, tools that edit files, sandboxed shell execution,
MCP, native Termux binary, feature parity with the eight source projects.
Do not claim the entire repository is building until each subsystem is reconciled and tested.

## Licensing
Preserve all original project licenses, notices, and third-party notices. An upstream source audit is still required.
