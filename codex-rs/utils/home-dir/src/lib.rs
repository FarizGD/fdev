use codex_utils_absolute_path::AbsolutePathBuf;
use dirs::home_dir;
use std::path::PathBuf;

/// Returns the FDev configuration root. FDEV_HOME takes precedence over the
/// legacy CODEX_HOME override; otherwise new installations use ~/.fdev.
/// Explicit overrides must name existing directories and are canonicalized.
/// The default directory is created by the existing configuration loader.
// @spec FDEV-002 — One configuration root with a legacy override
pub fn find_codex_home() -> std::io::Result<AbsolutePathBuf> {
    let fdev_home_env = std::env::var("FDEV_HOME").ok();
    let codex_home_env = std::env::var("CODEX_HOME").ok();
    find_home_from_env(fdev_home_env.as_deref(), codex_home_env.as_deref())
}

#[cfg(test)]
fn find_codex_home_from_env(codex_home_env: Option<&str>) -> std::io::Result<AbsolutePathBuf> {
    find_home_from_env(None, codex_home_env)
}

fn find_home_from_env(
    fdev_home_env: Option<&str>,
    codex_home_env: Option<&str>,
) -> std::io::Result<AbsolutePathBuf> {
    let fdev_home_env = fdev_home_env.filter(|value| !value.is_empty());
    let codex_home_env = codex_home_env.filter(|value| !value.is_empty());
    let env_name = if fdev_home_env.is_some() {
        "FDEV_HOME"
    } else {
        "CODEX_HOME"
    };
    let selected_home = fdev_home_env.or(codex_home_env);
    // Honor the `CODEX_HOME` environment variable when it is set to allow users
    // (and tests) to override the default location.
    match selected_home {
        Some(val) => {
            let path = PathBuf::from(val);
            let metadata = std::fs::metadata(&path).map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("{env_name} points to {val:?}, but that path does not exist"),
                ),
                _ => std::io::Error::new(
                    err.kind(),
                    format!("failed to read {env_name} {val:?}: {err}"),
                ),
            })?;

            if !metadata.is_dir() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("{env_name} points to {val:?}, but that path is not a directory"),
                ))
            } else {
                let canonical = path.canonicalize().map_err(|err| {
                    std::io::Error::new(
                        err.kind(),
                        format!("failed to canonicalize {env_name} {val:?}: {err}"),
                    )
                })?;
                AbsolutePathBuf::from_absolute_path(canonical)
            }
        }
        None => {
            let mut p = home_dir().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find home directory",
                )
            })?;
            p.push(".fdev");
            AbsolutePathBuf::from_absolute_path(p)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::find_codex_home_from_env;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use dirs::home_dir;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::io::ErrorKind;
    use tempfile::TempDir;

    #[test]
    fn find_codex_home_env_missing_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let missing = temp_home.path().join("missing-codex-home");
        let missing_str = missing
            .to_str()
            .expect("missing codex home path should be valid utf-8");

        let err = find_codex_home_from_env(Some(missing_str)).expect_err("missing CODEX_HOME");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert!(
            err.to_string().contains("CODEX_HOME"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_codex_home_env_file_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let file_path = temp_home.path().join("codex-home.txt");
        fs::write(&file_path, "not a directory").expect("write temp file");
        let file_str = file_path
            .to_str()
            .expect("file codex home path should be valid utf-8");

        let err = find_codex_home_from_env(Some(file_str)).expect_err("file CODEX_HOME");
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert!(
            err.to_string().contains("not a directory"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_codex_home_env_valid_directory_canonicalizes() {
        let temp_home = TempDir::new().expect("temp home");
        let temp_str = temp_home
            .path()
            .to_str()
            .expect("temp codex home path should be valid utf-8");

        let resolved = find_codex_home_from_env(Some(temp_str)).expect("valid CODEX_HOME");
        let expected = temp_home
            .path()
            .canonicalize()
            .expect("canonicalize temp home");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_codex_home_without_env_uses_default_home_dir() {
        let resolved =
            find_codex_home_from_env(/*codex_home_env*/ None).expect("default CODEX_HOME");
        let mut expected = home_dir().expect("home dir");
        expected.push(".fdev");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    // @spec FDEV-002 — Explicit FDev home takes precedence without fallback
    #[test]
    fn fdev_home_overrides_legacy_home() {
        let fdev = TempDir::new().expect("FDev directory");
        let legacy = TempDir::new().expect("legacy directory");
        let resolved = super::find_home_from_env(fdev.path().to_str(), legacy.path().to_str())
            .expect("resolve FDev home");
        assert_eq!(resolved.as_path(), fdev.path().canonicalize().unwrap());
    }

    #[test]
    fn invalid_fdev_home_does_not_fall_back_to_legacy() {
        let legacy = TempDir::new().expect("legacy directory");
        let missing = legacy.path().join("missing");
        let err = super::find_home_from_env(missing.to_str(), legacy.path().to_str())
            .expect_err("invalid FDev override must fail");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert!(err.to_string().contains("FDEV_HOME"));
    }

    #[test]
    fn empty_fdev_home_uses_legacy_override() {
        let legacy = TempDir::new().expect("legacy directory");
        let resolved = super::find_home_from_env(Some(""), legacy.path().to_str())
            .expect("resolve legacy home");
        assert_eq!(resolved.as_path(), legacy.path().canonicalize().unwrap());
    }
}
