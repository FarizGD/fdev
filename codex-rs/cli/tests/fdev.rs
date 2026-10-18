use anyhow::Result;
use predicates::str::contains;

// @spec FDEV-001 — One Codex-based executable
#[test]
fn fdev_help_and_version_identify_the_fork() -> Result<()> {
    let binary = codex_utils_cargo_bin::cargo_bin("fdev")?;
    assert_cmd::Command::new(&binary)
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("FDev"))
        .stdout(contains("fdev [OPTIONS]"));
    assert_cmd::Command::new(binary)
        .arg("--version")
        .assert()
        .success()
        .stdout(contains("fdev "));
    Ok(())
}

// @spec FDEV-003 — Provider setup is consumed by the existing configuration loader
#[test]
fn provider_config_outputs_runtime_configuration_without_secrets() -> Result<()> {
    let home = tempfile::TempDir::new()?;
    let output = assert_cmd::Command::new(codex_utils_cargo_bin::cargo_bin("fdev")?)
        .env("FDEV_HOME", home.path())
        .env("TEST_FDEV_API_KEY", "sentinel-do-not-print")
        .args([
            "provider",
            "config",
            "--kind",
            "openai-compatible",
            "--model",
            "local-model",
            "--base-url",
            "http://127.0.0.1:8080/v1",
            "--api-key-env",
            "TEST_FDEV_API_KEY",
            "--context-window",
            "8192",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output)?;
    let config: codex_config::config_toml::ConfigToml = toml::from_str(&text)?;
    assert_eq!(config.model.as_deref(), Some("local-model"));
    let provider = &config.model_providers["fdev-custom"];
    assert_eq!(provider.env_key.as_deref(), Some("TEST_FDEV_API_KEY"));
    assert!(!text.contains("sentinel-do-not-print"));
    assert!(!home.path().join("config.toml").exists());
    Ok(())
}

// Exercise the actual CLI, provider transport, tool dispatcher, and filesystem.
// Only the model service is mocked. This is not a real model or sandbox test.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn responses_stream_drives_file_edits_and_preapproved_shell() -> Result<()> {
    use serde_json::json;
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::time::Duration;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;
    use wiremock::matchers::method;
    use wiremock::matchers::path;

    let server = MockServer::start().await;
    let home = tempfile::TempDir::new()?;
    let workspace = tempfile::TempDir::new()?;
    std::fs::write(workspace.path().join("existing.txt"), "before\n")?;
    let sequence = Arc::new(AtomicUsize::new(0));
    let requests = Arc::clone(&sequence);
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(move |_: &wiremock::Request| {
            let turn = requests.fetch_add(1, Ordering::SeqCst);
            let item = match turn {
                0 => json!({
                    "type": "custom_tool_call", "name": "apply_patch", "call_id": "patch-1",
                    "input": "*** Begin Patch\n*** Add File: created.txt\n+created by the agent\n*** Update File: existing.txt\n@@\n-before\n+after\n*** End Patch\n"
                }),
                1 => json!({
                    "type": "function_call", "name": "exec_command", "call_id": "shell-1",
                    "arguments": json!({"cmd": if cfg!(windows) { "Get-Content created.txt; Write-Output fdev-shell-ok" } else { "cat created.txt; echo fdev-shell-ok" }, "max_output_tokens": 200}).to_string()
                }),
                _ => json!({
                    "type": "message", "role": "assistant", "id": "message-done",
                    "content": [{"type": "output_text", "text": "fdev-smoke-complete"}]
                }),
            };
            let events = [
                json!({"type": "response.created", "response": {"id": format!("response-{turn}")}}),
                json!({"type": "response.output_item.done", "item": item}),
                json!({"type": "response.completed", "response": {"id": format!("response-{turn}"), "usage": {"input_tokens": 1, "output_tokens": 1, "total_tokens": 2}}}),
            ];
            let body = events.iter().map(|event| format!("data: {event}\n\n")).collect::<String>();
            ResponseTemplate::new(200).insert_header("content-type", "text/event-stream").set_body_string(body)
        })
        .expect(3)
        .mount(&server)
        .await;
    let provider = format!(
        "model_providers.smoke={{name=\"Local test fixture\",base_url=\"{}/v1\",wire_api=\"responses\",requires_openai_auth=false,supports_websockets=false}}",
        server.uri()
    );
    // Explicit test-only permission grant: the model fixture emits only the
    // fixed patch and read/echo command above, inside a disposable directory.
    let output = assert_cmd::Command::new(codex_utils_cargo_bin::cargo_bin("fdev")?)
        .env("FDEV_HOME", home.path())
        .env("CODEX_HOME", home.path())
        .env_remove("OPENAI_API_KEY")
        .args([
            "exec",
            "--skip-git-repo-check",
            "--json",
            "--sandbox",
            "danger-full-access",
        ])
        .arg("-C")
        .arg(workspace.path())
        .args([
            "-c",
            &provider,
            "-c",
            "model_provider=\"smoke\"",
            "-c",
            "model=\"gpt-5.1-codex\"",
            "-c",
            "features.code_mode=false",
            "-c",
            "features.unified_exec=true",
            "-c",
            "check_for_update_on_startup=false",
        ])
        .arg("Apply the test patch, read the created file with the shell, and finish.")
        .timeout(Duration::from_secs(90))
        .assert()
        .success()
        .stdout(contains("fdev-smoke-complete"))
        .get_output()
        .clone();
    let captured = server.received_requests().await.unwrap();
    // A successful agent response does not imply that its tool calls succeeded.
    // Include tool results in failure output so CI reveals the actual cause,
    // rather than a context-free "No such file or directory" error.
    let diagnostic = || {
        let tool_results: Vec<_> = captured
            .iter()
            .filter_map(|request| request.body_json::<serde_json::Value>().ok())
            .filter_map(|body| {
                body.get("input")
                    .and_then(|input| input.as_array())
                    .cloned()
            })
            .flatten()
            .filter(|item| {
                matches!(
                    item.get("type").and_then(|kind| kind.as_str()),
                    Some("function_call_output" | "custom_tool_call_output")
                )
            })
            .collect();
        format!(
            "requests: {}; tool results: {tool_results:?}; CLI stdout: {}; CLI stderr: {}",
            captured.len(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    };
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("created.txt"))
            .ok()
            .as_deref(),
        Some("created by the agent\n"),
        "{}",
        diagnostic()
    );
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("existing.txt"))
            .ok()
            .as_deref(),
        Some("after\n"),
        "{}",
        diagnostic()
    );
    let last: serde_json::Value = captured.last().unwrap().body_json()?;
    let shell_output = last["input"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["type"] == "function_call_output" && item["call_id"] == "shell-1")
        .expect("shell result must be sent back to the provider");
    let output = shell_output["output"].to_string();
    assert!(
        output.contains("created by the agent") && output.contains("fdev-shell-ok"),
        "{output}"
    );
    Ok(())
}
