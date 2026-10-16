//! Bounded, Git-aware file context inspired by Aider and Continue repository maps.
//! This is a file inventory, not a symbol ranker; it never reads file contents.

use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

// @spec FDEV-005 — Bounded repository context without reading source files
pub(crate) fn tracked_file_map(path: &Path, max_files: usize) -> io::Result<String> {
    if !(1..=1000).contains(&max_files) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "file limit must be between 1 and 1000",
        ));
    }
    let mut child = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["ls-files", "--cached", "--deduplicate", "-z", "--"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let result = (|| {
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("missing Git output pipe"))?;
        let mut reader = BufReader::new(stdout);
        let mut map = String::from("# Git-tracked files (names only)\n\n");
        let mut count = 0;
        let mut truncated = false;
        loop {
            let mut name = Vec::new();
            let length = reader.by_ref().take(16_385).read_until(0, &mut name)?;
            if length == 0 {
                break;
            }
            if name.last() != Some(&0) || length > 16_384 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid or overlong Git filename",
                ));
            }
            name.pop();
            let escaped = String::from_utf8_lossy(&name).escape_debug().to_string();
            if count == max_files || map.len() + escaped.len() + 2 > 65_536 {
                truncated = true;
                break;
            }
            map.push_str("- ");
            map.push_str(&escaped);
            map.push('\n');
            count += 1;
        }
        if truncated {
            map.push_str(&format!("\n[map truncated after {count} files]\n"));
        }
        Ok((map, truncated))
    })();
    // Always reap Git, including when the bounded reader stops before EOF.
    // Otherwise large repositories could leave a child blocked on a full pipe.
    if !matches!(&result, Ok((_, false))) {
        let _ = child.kill();
    }
    let status = child.wait()?;
    let (map, truncated) = result?;
    if !truncated && !status.success() {
        return Err(io::Error::other(
            "git ls-files failed; select an accessible Git working directory",
        ));
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn repository() -> TempDir {
        let dir = TempDir::new().unwrap();
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .arg(dir.path())
                .status()
                .unwrap()
                .success()
        );
        for name in ["a.rs", "space name.rs", "z.rs"] {
            fs::write(dir.path().join(name), "secret source contents").unwrap();
        }
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(dir.path())
                .args(["add", "--", "a.rs", "space name.rs", "z.rs"])
                .status()
                .unwrap()
                .success()
        );
        fs::write(dir.path().join("untracked-secret"), "private").unwrap();
        dir
    }

    // @spec FDEV-005 — Bounded repository context from Git's tracked file inventory
    #[test]
    fn inventory_includes_tracked_names_without_contents_or_untracked_files() {
        let repo = repository();
        let map = tracked_file_map(repo.path(), 10).unwrap();
        assert!(map.contains("a.rs") && map.contains("space name.rs") && map.contains("z.rs"));
        assert!(!map.contains("secret") && !map.contains("private"));
        assert!(!map.contains("truncated"));
    }

    #[test]
    fn limit_is_enforced_and_reported() {
        let repo = repository();
        let map = tracked_file_map(repo.path(), 1).unwrap();
        assert!(map.contains("a.rs"));
        assert!(!map.contains("z.rs"));
        assert!(map.contains("truncated"));
        assert!(tracked_file_map(repo.path(), 0).is_err());
        assert!(tracked_file_map(repo.path(), 1001).is_err());
    }

    #[test]
    fn a_non_repository_fails_instead_of_claiming_empty_context() {
        let dir = TempDir::new().unwrap();
        assert!(tracked_file_map(dir.path(), 10).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn filenames_cannot_inject_terminal_controls() {
        let repo = repository();
        let name = "escape\x1b[31m\n.rs";
        fs::write(repo.path().join(name), "hidden").unwrap();
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(repo.path())
                .args(["add", "--", name])
                .status()
                .unwrap()
                .success()
        );
        let map = tracked_file_map(repo.path(), 10).unwrap();
        assert!(!map.contains('\x1b'));
        assert!(map.contains("\\n.rs"));
    }
}
