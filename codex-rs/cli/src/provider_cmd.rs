//! FDev provider setup. Keep inference in Codex's existing provider/runtime stack.
use anyhow::Result;
use anyhow::bail;
use clap::Args;
use clap::Subcommand;
use clap::ValueEnum;
use url::Url;

#[derive(Debug, Args)]
pub(crate) struct ProviderCommand {
    #[command(subcommand)]
    action: ProviderAction,
}

#[derive(Debug, Subcommand)]
enum ProviderAction {
    /// Show protocol support; this does not contact a provider.
    List,
    /// Print a validated config.toml fragment; review it before saving.
    Config(ProviderConfig),
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ProviderKind {
    Openai,
    Ollama,
    Openrouter,
    OpenaiCompatible,
    Gemini,
    Anthropic,
}

#[derive(Debug, Args)]
struct ProviderConfig {
    #[arg(long, value_enum)]
    kind: ProviderKind,
    #[arg(long)]
    model: String,
    /// API base URL, excluding /responses. HTTP is allowed only on loopback.
    #[arg(long)]
    base_url: Option<String>,
    /// Name of the environment variable containing the key, never the key itself.
    #[arg(long)]
    api_key_env: Option<String>,
    /// Context size in tokens; use the model's actual limit.
    #[arg(long)]
    context_window: Option<u64>,
}

impl ProviderCommand {
    pub(crate) fn run(self) -> Result<()> {
        match self.action {
            ProviderAction::List => println!(
                "OpenAI: Responses API\nOllama: requires /v1/responses support\nOpenRouter: Responses-compatible endpoint (experimental)\nOpenAI-compatible/self-hosted: requires Responses API\nGemini native: not implemented\nAnthropic native: not implemented\n\nStreaming and tools depend on the endpoint and chosen model."
            ),
            ProviderAction::Config(config) => println!("{}", config.render()?),
        }
        Ok(())
    }
}

// @spec FDEV-003 — Validated provider configuration without storing credentials
impl ProviderConfig {
    fn render(&self) -> Result<String> {
        let (name, default_url, default_key) = match self.kind {
            ProviderKind::Openai => (
                "OpenAI API",
                Some("https://api.openai.com/v1"),
                Some("OPENAI_API_KEY"),
            ),
            ProviderKind::Ollama => ("Ollama", Some("http://127.0.0.1:11434/v1"), None),
            ProviderKind::Openrouter => (
                "OpenRouter",
                Some("https://openrouter.ai/api/v1"),
                Some("OPENROUTER_API_KEY"),
            ),
            ProviderKind::OpenaiCompatible => ("Responses-compatible server", None, None),
            ProviderKind::Gemini | ProviderKind::Anthropic => bail!(
                "Native Gemini and Anthropic protocols are not implemented. Use a separately configured Responses-compatible gateway with --kind openai-compatible, or choose a supported provider."
            ),
        };
        if self.model.trim().is_empty() || self.model.chars().any(char::is_control) {
            bail!("model must be nonempty and contain no control characters");
        }
        if self
            .context_window
            .is_some_and(|limit| limit == 0 || limit > i64::MAX as u64)
        {
            bail!("context window must be a positive 64-bit signed integer");
        }
        let base_url = self
            .base_url
            .as_deref()
            .or(default_url)
            .ok_or_else(|| anyhow::anyhow!("--base-url is required for a custom provider"))?;
        // Do not echo invalid URLs: they may contain accidentally pasted credentials.
        let url = Url::parse(base_url).map_err(|_| anyhow::anyhow!("invalid API base URL"))?;
        if !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            bail!(
                "API base URL must not contain credentials, a query, or a fragment; use --api-key-env"
            );
        }
        let loopback = match url.host() {
            Some(url::Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
            Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
            Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
            None => false,
        };
        if url.host().is_none()
            || !(url.scheme() == "https" || (url.scheme() == "http" && loopback))
        {
            bail!("use HTTPS for remote providers; HTTP is allowed only on a loopback endpoint");
        }
        let path = url.path().trim_end_matches('/');
        if path.ends_with("/responses")
            || path.ends_with("/chat/completions")
            || path.ends_with("/messages")
        {
            bail!("use the API base URL without /responses, /chat/completions, or /messages");
        }
        let key = self.api_key_env.as_deref().or(default_key);
        if let Some(key) = key {
            let mut chars = key.chars();
            if !chars
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                bail!(
                    "API key environment variable name must use letters, digits, and underscores, and must not start with a digit"
                );
            }
        }
        let mut provider = toml::Table::new();
        provider.insert("name".into(), name.into());
        provider.insert("base_url".into(), url.as_str().trim_end_matches('/').into());
        provider.insert("wire_api".into(), "responses".into());
        provider.insert("requires_openai_auth".into(), false.into());
        provider.insert("supports_websockets".into(), false.into());
        if let Some(key) = key {
            provider.insert("env_key".into(), key.into());
        }
        let mut capabilities = toml::Table::new();
        capabilities.insert("remote_compaction".into(), "unsupported".into());
        capabilities.insert("external_web_access".into(), false.into());
        provider.insert("capabilities".into(), capabilities.into());
        let mut providers = toml::Table::new();
        providers.insert("fdev-custom".into(), provider.into());
        let mut config = toml::Table::new();
        config.insert("model".into(), self.model.trim().into());
        config.insert("model_provider".into(), "fdev-custom".into());
        if let Some(limit) = self.context_window {
            config.insert("model_context_window".into(), (limit as i64).into());
        }
        config.insert("model_providers".into(), providers.into());
        Ok(toml::to_string_pretty(&config)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_config::config_toml::ConfigToml;

    fn config(kind: ProviderKind, base_url: Option<&str>) -> ProviderConfig {
        ProviderConfig {
            kind,
            model: "custom/model\"name".into(),
            base_url: base_url.map(str::to_owned),
            api_key_env: None,
            context_window: Some(32768),
        }
    }

    #[test]
    fn generated_profiles_are_accepted_by_the_runtime_configuration() -> Result<()> {
        for kind in [
            ProviderKind::Openai,
            ProviderKind::Ollama,
            ProviderKind::Openrouter,
            ProviderKind::OpenaiCompatible,
        ] {
            let input = config(kind, Some("http://[::1]:8080/v1/"));
            let rendered = input.render()?;
            let parsed: ConfigToml = toml::from_str(&rendered)?;
            assert_eq!(parsed.model.as_deref(), Some(input.model.as_str()));
            assert_eq!(parsed.model_context_window, Some(32768));
            let provider = &parsed.model_providers[parsed.model_provider.as_ref().unwrap()];
            assert_eq!(provider.base_url.as_deref(), Some("http://[::1]:8080/v1"));
            assert!(!provider.requires_openai_auth);
            assert!(!provider.supports_websockets);
        }
        Ok(())
    }

    #[test]
    fn rejects_unsafe_endpoints_without_echoing_credentials() {
        for endpoint in [
            "https://user:secret@example.com/v1",
            "https://example.com/v1?key=secret",
            "http://example.com/v1",
            "file:///secret",
            "https://example.com/v1/responses",
            "https://example.com/v1/chat/completions",
        ] {
            let err = config(ProviderKind::OpenaiCompatible, Some(endpoint))
                .render()
                .unwrap_err()
                .to_string();
            assert!(!err.contains("secret"));
        }
    }

    #[test]
    fn rejects_invalid_models_keys_limits_and_missing_endpoints() {
        let mut input = config(ProviderKind::Openai, None);
        input.model = " ".into();
        assert!(input.render().is_err());
        input.model = "model".into();
        input.api_key_env = Some("sk-secret-value".into());
        assert!(input.render().is_err());
        input.api_key_env = None;
        input.context_window = Some(0);
        assert!(input.render().is_err());
        assert!(
            config(ProviderKind::OpenaiCompatible, None)
                .render()
                .is_err()
        );
    }

    #[test]
    fn native_protocols_are_explicitly_unsupported() {
        for kind in [ProviderKind::Anthropic, ProviderKind::Gemini] {
            assert!(
                config(kind, None)
                    .render()
                    .unwrap_err()
                    .to_string()
                    .contains("not implemented")
            );
        }
    }
}
