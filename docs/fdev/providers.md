# Provider configuration

The inference runtime currently speaks the OpenAI Responses protocol. A server
advertising only Chat Completions is not sufficient. Native Anthropic Messages
and Gemini protocols are not implemented. A Responses-compatible gateway is a
separate service; FDev does not silently start or supply one.

After a verified build, inspect supported setup templates with:

```sh
fdev provider list
fdev provider config --kind openai --model YOUR_MODEL
fdev provider config --kind ollama --model YOUR_INSTALLED_MODEL --context-window 32768
fdev provider config --kind openrouter --model PROVIDER/MODEL
fdev provider config --kind openai-compatible --model YOUR_MODEL \
  --base-url https://inference.example/v1 --api-key-env MY_MODEL_API_KEY
```

These commands print TOML for review; they do not write configuration or make
network requests. Merge the output into `~/.fdev/config.toml` without replacing
unrelated settings. Set the referenced API-key environment variable outside
source control. The default references are OPENAI_API_KEY and
OPENROUTER_API_KEY; the helper never reads their values. Local endpoints can
omit authentication. Remote endpoints must use HTTPS; loopback HTTP is allowed.
Credentials, query strings, and fragments in URLs are rejected without echoing
them. Use a custom environment-variable name, not the API key as an argument.

FDEV_HOME takes precedence over CODEX_HOME. Both explicit overrides must point
to an existing directory. Without an override, FDev uses ~/.fdev. Existing
Codex configuration formats are retained. Select an old configuration explicitly
with CODEX_HOME; FDev does not copy credentials or merge histories automatically.
Use the model's actual context window, not a guessed limit.

The templates disable WebSockets, provider-side compaction, and live hosted
web search for compatibility. Inference streams over HTTP SSE through the
existing Codex transport. Tool support still depends on the chosen model and
server. OpenRouter configuration remains experimental until exercised against
a real account. Ollama must expose a compatible `/v1/responses` endpoint.
See the [OpenRouter Responses reference](https://openrouter.ai/docs/api/api-reference/responses/create-responses)
and [Ollama compatibility documentation](https://github.com/ollama/ollama/blob/main/docs/api/openai-compatibility.mdx).

The existing TUI `/model` command remains the model-selection interface. Custom
model catalogs and capability overrides continue to use the upstream TOML
schema. Interactive provider switching and new native protocol adapters remain
future work.

Live provider validation is deferred at the user's request until credentials
are available. Mock streaming tests are not evidence of real model inference.
