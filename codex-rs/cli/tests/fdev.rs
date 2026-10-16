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
