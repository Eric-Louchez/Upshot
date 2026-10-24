//! Loads the spec files from the working tree, or (best effort) from a Git revision.

use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::model::{SpecFile, SpecSet};

pub const LOG_FOLDER: &str = "spec/log";
pub const INTERPRETATIONS_PATH: &str = "spec/interpretations.md";
pub const CURRENT_FOLDER: &str = "spec/current";

fn read_text(path: &Path) -> io::Result<String> {
    let text = fs::read_to_string(path)?;
    Ok(text
        .strip_prefix('\u{feff}')
        .map(str::to_owned)
        .unwrap_or(text))
}

/// The `*.md` files of a folder of the repository, in ordinal order of their names; none when it doesn't exist.
#[allow(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "spec files are named *.md exactly"
)]
fn folder(repository: &Path, relative: &str) -> io::Result<Vec<SpecFile>> {
    let dir = repository.join(relative);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() && !entry.path().is_file() {
            continue;
        }
        if let Some(name) = entry.file_name().to_str()
            && name.ends_with(".md")
        {
            names.push(name.to_owned());
        }
    }
    names.sort();
    names
        .into_iter()
        .map(|name| {
            Ok(SpecFile::new(
                format!("{relative}/{name}"),
                read_text(&dir.join(&name))?,
            ))
        })
        .collect()
}

/// The spec files of the working tree at `repository`.
///
/// # Errors
/// A spec file or folder that exists but can't be read (or isn't UTF-8).
pub fn from_working_tree(repository: &Path) -> io::Result<SpecSet> {
    let interpretations = repository.join(INTERPRETATIONS_PATH);
    Ok(SpecSet {
        logs: folder(repository, LOG_FOLDER)?,
        interpretations: if interpretations.is_file() {
            Some(SpecFile::new(
                INTERPRETATIONS_PATH,
                read_text(&interpretations)?,
            ))
        } else {
            None
        },
        current: folder(repository, CURRENT_FOLDER)?,
    })
}

/// The log and interpretation files at `revision` (e.g. `origin/main`), read with git; `None` when git or the
/// revision isn't there. It never fetches: what was fetched last is what counts.
#[must_use]
#[allow(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "spec files are named *.md exactly"
)]
pub fn from_git(repository: &Path, revision: &str) -> Option<SpecSet> {
    git(
        repository,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{revision}^{{commit}}"),
        ],
    )?;
    let mut logs = Vec::new();
    if let Some(names) = git(
        repository,
        &[
            "ls-tree",
            "--name-only",
            &format!("{revision}:{LOG_FOLDER}"),
        ],
    ) {
        for name in names.lines().filter(|n| n.ends_with(".md")) {
            let path = format!("{revision}:{LOG_FOLDER}/{name}");
            if let Some(text) = git(repository, &["show", &path]) {
                logs.push(SpecFile::new(path, text));
            }
        }
    }
    let path = format!("{revision}:{INTERPRETATIONS_PATH}");
    let interpretations = git(repository, &["show", &path]).map(|text| SpecFile::new(path, text));
    Some(SpecSet {
        logs,
        interpretations,
        current: Vec::new(),
    })
}

/// git's standard output, or `None` when git can't be run or fails.
fn git(repository: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repository)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("upshot-spec-tools-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("a scratch folder");
        dir
    }

    #[test]
    fn the_working_tree_is_read_in_order() {
        let dir = scratch("tree");
        fs::create_dir_all(dir.join("spec/log")).expect("log folder");
        fs::write(dir.join("spec/log/2026-10.md"), "\u{feff}# B\n").expect("write");
        fs::write(dir.join("spec/log/2026-09.md"), "# A\n").expect("write");
        fs::write(dir.join("spec/log/notes.txt"), "x").expect("write");
        fs::write(dir.join("spec/interpretations.md"), "| J1 | L1 | x |\n").expect("write");
        let spec = from_working_tree(&dir).expect("readable");
        let paths: Vec<&str> = spec.logs.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, ["spec/log/2026-09.md", "spec/log/2026-10.md"]);
        assert_eq!(spec.logs[1].text, "# B\n");
        assert!(spec.interpretations.is_some());
        assert!(spec.current.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn no_git_revision_gives_none() {
        let dir = scratch("git");
        assert_eq!(from_git(&dir, "origin/main"), None);
        let _ = fs::remove_dir_all(&dir);
    }
}
